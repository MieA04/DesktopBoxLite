use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::Mutex;
use std::thread;

use tauri::Emitter;

use crate::icon_cache::IconCache;
use crate::icons;

/// Holds the current desktop icon state and manages background extraction.
///
/// Registered as Tauri Managed State (`app.manage(...)`).
pub struct IconState {
    /// Fingerprint of the last known icon set.
    pub fingerprint: Mutex<String>,
    /// Whether an extraction is currently in progress.
    pub extraction_running: AtomicBool,
    /// Sender used to cancel a running extraction.
    cancel_tx: Mutex<Option<mpsc::Sender<()>>>,
    /// Disk cache for extracted icons.
    pub cache: IconCache,
}

impl IconState {
    pub fn new(config_path: &PathBuf) -> Self {
        let cache = IconCache::new(config_path);
        cleanup_stale_cache(&cache);

        IconState {
            fingerprint: Mutex::new(String::new()),
            extraction_running: AtomicBool::new(false),
            cancel_tx: Mutex::new(None),
            cache,
        }
    }

    /// Requests a full icon refresh in a background thread.
    ///
    /// 1. Cancels any running extraction.
    /// 2. Scans desktop → extracts each icon (cache-aware) → emits result via event.
    /// 3. On completion emits `"icons-ready"` with payload `Vec<DesktopIcon>`.
    /// 4. If cancelled mid-way, aborts silently (no event emitted).
    pub fn request_refresh(&self, app: tauri::AppHandle) {
        // Cancel any running extraction
        if let Ok(mut guard) = self.cancel_tx.lock() {
            if let Some(sender) = guard.take() {
                let _ = sender.send(());
            }
        }

        self.extraction_running.store(true, Ordering::SeqCst);

        let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
        *self.cancel_tx.lock().unwrap() = Some(cancel_tx);

        let cache_dir = self.cache.cache_dir().to_path_buf();
        let app_clone = app.clone();

        thread::spawn(move || {
            log::info!("Icon extraction started in background thread");

            // Scan metadata (lightweight)
            let metas = match icons::scan_icons_meta() {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to scan icon metas: {}", e);
                    let _ = app_clone.emit("icons-error", format!("Scan failed: {}", e));
                    return;
                }
            };

            let cache = IconCache::from_dir(&cache_dir);
            let mut results = Vec::with_capacity(metas.len());
            let mut cancelled = false;

            for meta in &metas {
                // Check cancellation signal
                match cancel_rx.try_recv() {
                    Ok(()) | Err(TryRecvError::Disconnected) => {
                        log::info!("Icon extraction cancelled (newer request pending)");
                        cancelled = true;
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                }

                let path = PathBuf::from(&meta.path);

                // Cache-aware extraction (now also returns click_count)
                let (icon_data, click_count) = cache.get(&path, meta.mtime).unwrap_or_else(|| {
                    let data = icons::extract_file_icon(&path);
                    if !data.is_empty() {
                        // Preserve existing click count across re-extraction
                        let old_count = cache.read_click_count(&path);
                        cache.set(&path, meta.mtime, &data, old_count);
                        (data, old_count)
                    } else {
                        (String::new(), 0)
                    }
                });

                results.push(icons::DesktopIcon {
                    name: meta.name.clone(),
                    path: meta.path.clone(),
                    icon_data,
                    is_shortcut: meta.is_shortcut,
                    click_count,
                });
            }

            if cancelled {
                return; // No event — the next request will handle it
            }

            log::info!("Icon extraction completed: {} icons", results.len());
            let _ = app_clone.emit("icons-ready", results);
        });
    }
}

/// Removes cache entries for files that no longer exist on the desktop.
fn cleanup_stale_cache(cache: &IconCache) {
    let current_paths: std::collections::HashSet<String> =
        match icons::scan_icons_meta() {
            Ok(metas) => metas.into_iter().map(|m| m.path).collect(),
            Err(_) => return,
        };

    let removed = cache.remove_stale(&current_paths);
    if removed > 0 {
        log::info!("Cleaned up {} stale icon cache entries", removed);
    }
}
