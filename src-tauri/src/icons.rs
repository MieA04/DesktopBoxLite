use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Information about a single desktop icon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopIcon {
    /// Display name (without extension).
    pub name: String,
    /// Full path to the file/shortcut.
    pub path: String,
    /// Base64-encoded PNG icon data (empty string if extraction failed).
    pub icon_data: String,
    /// Whether this item is a shortcut (.lnk).
    pub is_shortcut: bool,
    /// How many times the user has clicked this icon (persisted in cache).
    pub click_count: u64,
}

fn user_desktop_path() -> PathBuf {
    dirs_next().unwrap_or_else(|| PathBuf::from("C:\\"))
}

fn public_desktop_path() -> PathBuf {
    PathBuf::from(r"C:\Users\Public\Desktop")
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .ok()
        .map(|p| PathBuf::from(p).join("Desktop"))
}

fn strip_extension(filename: &str) -> String {
    let extensions_to_strip = ["lnk", "exe", "url", "txt", "doc", "docx", "pdf"];
    if let Some(dot) = filename.rfind('.') {
        let ext = &filename[dot + 1..];
        if extensions_to_strip.contains(&ext.to_lowercase().as_str()) {
            return filename[..dot].to_string();
        }
    }
    filename.to_string()
}

fn resolve_lnk_target(path: &PathBuf) -> Option<String> {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
}

// ── Lightweight metadata for change detection ──────────

/// Lightweight icon metadata (no icon data), used for change-detection polling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IconMeta {
    pub name: String,
    pub path: String,
    pub is_shortcut: bool,
    /// File modification timestamp (UNIX seconds).
    pub mtime: u64,
}

/// Scans desktop directories and returns lightweight metadata for each icon.
/// This is intentionally MUCH faster than `scan_desktop_icons()` — no icon extraction.
pub fn scan_icons_meta() -> Result<Vec<IconMeta>, String> {
    let mut metas: Vec<IconMeta> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let user_path = user_desktop_path();
    if user_path.exists() {
        scan_directory_meta(&user_path, &mut metas, &mut seen)?;
    }

    let public_path = public_desktop_path();
    if public_path.exists() {
        scan_directory_meta(&public_path, &mut metas, &mut seen)?;
    }

    metas.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(metas)
}

fn scan_directory_meta(
    dir: &PathBuf,
    metas: &mut Vec<IconMeta>,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let file_name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        if file_name.starts_with('.') || file_name.eq_ignore_ascii_case("desktop.ini") {
            continue;
        }

        let display_name = strip_extension(&file_name);
        let (dedup_key, is_shortcut) = if path.extension().map_or(false, |e| {
            e.eq_ignore_ascii_case("lnk")
        }) {
            let target = resolve_lnk_target(&path).unwrap_or_else(|| file_name.clone());
            (target, true)
        } else {
            (path.to_string_lossy().to_string(), false)
        };

        if !seen.insert(dedup_key) {
            continue;
        }

        let mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        metas.push(IconMeta {
            name: display_name,
            path: path.to_string_lossy().to_string(),
            is_shortcut,
            mtime,
        });
    }

    Ok(())
}

/// Scans desktop icons and returns a fingerprint (hash) of the current state.
/// Used by `check_icons_changed` for lightweight change detection.
pub fn compute_fingerprint() -> Result<String, String> {
    let metas = scan_icons_meta()?;
    Ok(compute_fingerprint_from_metas(&metas))
}

/// Computes a fingerprint (hash) from a sorted list of icon metadata.
/// Used to detect whether the desktop icon set has changed.
pub fn compute_fingerprint_from_metas(metas: &[IconMeta]) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for meta in metas {
        meta.path.hash(&mut hasher);
        meta.mtime.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

// ── HD Icon Extraction (Windows only) ──────────────────
// Uses IShellItemImageFactory COM API to extract 256x256 HD icons.
// This API is available on Windows Vista+ and produces significantly
// sharper icons than the old SHGetFileInfoW (which maxes out at 48x48).

#[cfg(target_os = "windows")]
mod icon_extract {
    use std::os::windows::ffi::OsStrExt;
    use std::path::PathBuf;
    use std::ptr;

    use windows_sys::Win32::Graphics::Gdi::{
        GetDC, ReleaseDC, GetDIBits, DeleteObject, GetObjectW,
        BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };

    // ── COM types ───────────────────────────────────────

    type HRESULT = i32;

    #[repr(C)]
    struct GUID {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    // IID_IShellItemImageFactory: bcc18b79-ba16-442f-80c4-8a59c30c463b
    const IID_ISHELL_ITEM_IMAGE_FACTORY: GUID = GUID {
        data1: 0xbcc18b79,
        data2: 0xba16,
        data3: 0x442f,
        data4: [0x80, 0xc4, 0x8a, 0x59, 0xc3, 0x0c, 0x46, 0x3b],
    };

    // SIIGBF flags for IShellItemImageFactory::GetImage
    const SIIGBF_ICONONLY: u32 = 0x04;

    #[repr(C)]
    struct SIZE {
        cx: i32,
        cy: i32,
    }

    // COM vtable for IShellItemImageFactory (IUnknown + 1 method)
    #[repr(C)]
    struct IShellItemImageFactoryVtbl {
        query_interface: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            *const GUID,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT,
        add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
        release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
        get_image: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            SIZE,
            u32,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT,
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHCreateItemFromParsingName(
            pszPath: *const u16,
            pbc: *mut std::ffi::c_void,
            riid: *const GUID,
            ppv: *mut *mut std::ffi::c_void,
        ) -> HRESULT;
    }

    // ── Public entry point ──────────────────────────────

    /// Extracts a 256x256 HD icon for the given file path.
    /// Returns a base64-encoded PNG string, or empty string on failure.
    pub fn extract_file_icon(path: &PathBuf) -> String {
        unsafe { extract_via_shell_factory(path) }
    }

    unsafe fn extract_via_shell_factory(path: &PathBuf) -> String {
        // 1. Convert path to wide string
        let path_wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // 2. Create IShellItemImageFactory via SHCreateItemFromParsingName
        let mut factory: *mut std::ffi::c_void = ptr::null_mut();
        let hr = SHCreateItemFromParsingName(
            path_wide.as_ptr(),
            ptr::null_mut(),
            &IID_ISHELL_ITEM_IMAGE_FACTORY,
            &mut factory,
        );

        if hr < 0 || factory.is_null() {
            return String::new();
        }

        // 3. Get the vtable pointer
        let vtbl = *(factory as *mut *const IShellItemImageFactoryVtbl);

        // 4. Call GetImage(256x256, ICONONLY)
        let mut hbitmap: *mut std::ffi::c_void = ptr::null_mut();
        let desired_size = SIZE {
            cx: 256,
            cy: 256,
        };

        let hr = ((*vtbl).get_image)(
            factory,
            desired_size,
            SIIGBF_ICONONLY,
            &mut hbitmap,
        );

        // 5. Release the COM factory immediately (bitmap is independent)
        let _ = ((*vtbl).release)(factory);

        if hr < 0 || hbitmap.is_null() {
            return String::new();
        }

        // 6. Determine actual bitmap dimensions via GetObjectW
        let mut bm: BITMAP = std::mem::zeroed();
        let obj_result = GetObjectW(
            hbitmap as *mut std::ffi::c_void,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut _ as *mut std::ffi::c_void,
        );

        if obj_result == 0 {
            let _ = DeleteObject(hbitmap);
            return String::new();
        }

        let w = bm.bmWidth;
        let h = bm.bmHeight;

        if w <= 0 || h <= 0 {
            let _ = DeleteObject(hbitmap);
            return String::new();
        }

        // 7. Read pixel data via GetDIBits
        let screen_dc = GetDC(ptr::null_mut());
        if screen_dc.is_null() {
            let _ = DeleteObject(hbitmap);
            return String::new();
        }

        let mut bmp_info: BITMAPINFO = std::mem::zeroed();
        bmp_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmp_info.bmiHeader.biWidth = w;
        bmp_info.bmiHeader.biHeight = -h; // top-down DIB
        bmp_info.bmiHeader.biPlanes = 1;
        bmp_info.bmiHeader.biBitCount = 32;
        bmp_info.bmiHeader.biCompression = BI_RGB;

        let pixel_count = (w * h) as usize;
        let mut pixels: Vec<u8> = vec![0u8; pixel_count * 4];

        let dib_result = GetDIBits(
            screen_dc,
            hbitmap,
            0,
            h as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bmp_info,
            DIB_RGB_COLORS,
        );

        let _ = ReleaseDC(ptr::null_mut(), screen_dc);
        let _ = DeleteObject(hbitmap);

        if dib_result == 0 {
            return String::new();
        }

        // 8. BGRA → RGBA (swap R and B channels)
        for chunk in pixels.chunks_mut(4) {
            let b = chunk[0];
            let r = chunk[2];
            chunk[0] = r;
            chunk[2] = b;
        }

        // 9. Encode as PNG → base64
        type RgbaImage = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
        match RgbaImage::from_raw(w as u32, h as u32, pixels) {
            Some(img) => {
                let mut png_bytes: Vec<u8> = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                if img.write_with_encoder(encoder).is_ok() {
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD.encode(png_bytes)
                } else {
                    String::new()
                }
            }
            None => String::new(),
        }
    }
}

#[cfg(target_os = "windows")]
pub fn extract_file_icon(path: &PathBuf) -> String {
    icon_extract::extract_file_icon(path)
}

#[cfg(not(target_os = "windows"))]
pub fn extract_file_icon(_path: &PathBuf) -> String {
    String::new()
}
