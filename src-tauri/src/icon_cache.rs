use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache entry stored on disk.
#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    /// Original file path (for stale cleanup reverse lookup).
    path: String,
    /// File modification timestamp (seconds since UNIX epoch).
    mtime: u64,
    /// Base64-encoded PNG icon data.
    icon_data: String,
    /// How many times the user has clicked this icon.
    #[serde(default)]
    click_count: u64,
}

/// Manages a persistent on-disk icon cache.
///
/// **Directory**: `<exe_dir>/cache/`
///
/// **Key scheme**: Hash of the absolute file path (stable, not affected by mtime).
///
/// **Lookup**: Read file → compare mtime → return cached data if match.
///
/// **Invalidation**: When a file is deleted, `remove()` is called for its key.
/// When mtime changes, `get()` returns None → caller re-extracts → `set()` overwrites.
pub struct IconCache {
    cache_dir: PathBuf,
}

impl IconCache {
    /// Creates a new cache in `<config_parent>/cache/`.
    /// Creates the directory if it doesn't exist.
    pub fn new(config_path: &Path) -> Self {
        let cache_dir = config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("cache");

        let _ = std::fs::create_dir_all(&cache_dir);
        log::info!("Icon cache directory: {:?}", cache_dir);

        IconCache { cache_dir }
    }

    /// Returns the cached icon data and click count for `path` if it exists and its mtime matches.
    /// Returns `None` if the cache is missing, stale, or corrupted.
    pub fn get(&self, path: &Path, mtime: u64) -> Option<(String, u64)> {
        let cache_file = self.cache_path(path);
        if !cache_file.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cache_file).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        if entry.mtime == mtime {
            Some((entry.icon_data, entry.click_count))
        } else {
            // Stale cache — caller will re-extract
            log::debug!("Cache stale for {:?}: mtime mismatch", path);
            None
        }
    }

    /// Writes (or overwrites) the cache entry for `path`,
    /// including the given `click_count`.
    pub fn set(&self, path: &Path, mtime: u64, icon_data: &str, click_count: u64) {
        let cache_file = self.cache_path(path);
        let entry = CacheEntry {
            path: path.to_string_lossy().to_string(),
            mtime,
            icon_data: icon_data.to_string(),
            click_count,
        };

        match serde_json::to_string(&entry) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&cache_file, &json) {
                    log::warn!("Failed to write icon cache {:?}: {}", cache_file, e);
                }
            }
            Err(e) => log::warn!("Failed to serialize cache entry: {}", e),
        }
    }

    /// Increments the click count for `path` and returns the new count.
    /// Returns `Err` if no cache entry exists for the path (e.g. first extraction
    /// not yet completed).
    pub fn increment_click_count(&self, path: &Path) -> Result<u64, String> {
        let cache_file = self.cache_path(path);
        let content =
            std::fs::read_to_string(&cache_file).map_err(|e| format!("Cache read error: {}", e))?;
        let mut entry: CacheEntry = serde_json::from_str(&content)
            .map_err(|e| format!("Cache parse error: {}", e))?;
        entry.click_count += 1;
        let new_count = entry.click_count;
        let json =
            serde_json::to_string(&entry).map_err(|e| format!("Cache serialize error: {}", e))?;
        std::fs::write(&cache_file, &json).map_err(|e| format!("Cache write error: {}", e))?;
        Ok(new_count)
    }

    /// Reads the existing click count for `path` without checking mtime.
    /// Returns 0 if the cache entry doesn't exist (first-time extraction).
    pub fn read_click_count(&self, path: &Path) -> u64 {
        let cache_file = self.cache_path(path);
        if !cache_file.exists() {
            return 0;
        }
        std::fs::read_to_string(&cache_file)
            .ok()
            .and_then(|c| serde_json::from_str::<CacheEntry>(&c).ok())
            .map(|e| e.click_count)
            .unwrap_or(0)
    }

    /// Returns a reference to the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Creates a cache instance from an existing cache directory path.
    pub fn from_dir(dir: &Path) -> Self {
        IconCache {
            cache_dir: dir.to_path_buf(),
        }
    }

    /// Removes cache entries whose original file paths are no longer in `current_paths`.
    /// Returns the number of removed entries.
    #[allow(dead_code)]
    pub fn remove_stale(&self, current_paths: &std::collections::HashSet<String>) -> u32 {
        let mut removed = 0;
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let cache_file = entry.path();
                if !cache_file.is_file() {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&cache_file) {
                    if let Ok(cached) = serde_json::from_str::<CacheEntry>(&content) {
                        if !current_paths.contains(&cached.path) {
                            if std::fs::remove_file(&cache_file).is_ok() {
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }
        removed
    }

    /// Removes the cache entry for `path` (e.g. when the file has been deleted).
    #[allow(dead_code)]
    pub fn remove(&self, path: &Path) {
        let cache_file = self.cache_path(path);
        if cache_file.exists() {
            let _ = std::fs::remove_file(&cache_file);
        }
    }

    /// Clears the entire icon cache.
    pub fn clear_all(&self) {
        if let Err(e) = std::fs::remove_dir_all(&self.cache_dir) {
            log::warn!("Failed to clear icon cache: {}", e);
        }
        let _ = std::fs::create_dir_all(&self.cache_dir);
    }

    /// Builds the cache file path for a given absolute file path.
    fn cache_path(&self, path: &Path) -> PathBuf {
        let key = cache_key(path);
        self.cache_dir.join(key)
    }
}

/// Computes a stable hash string for an absolute file path.
/// Used as the cache file name.
fn cache_key(path: &Path) -> String {
    let canonical = if let Ok(p) = path.canonicalize() {
        p
    } else {
        path.to_path_buf()
    };
    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Extracts a UNIX-timestamp mtime from `std::fs::Metadata`.
/// Returns 0 if the mtime is unavailable (graceful fallback).
pub fn mtime_from_metadata(meta: &std::fs::Metadata) -> u64 {
    match meta.modified() {
        Ok(time) => {
            let duration = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            duration.as_secs()
        }
        Err(_) => 0,
    }
}
