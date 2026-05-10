use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use image::{imageops::FilterType, ImageFormat};

const ICON_SIZE: u32 = 256;

pub(super) fn read_source_app_icon_data_url(
    source_app_id: &str,
    desktop_entry: Option<&str>,
) -> String {
    let cache_key = format!(
        "{}\u{1f}{}",
        source_app_id.trim(),
        desktop_entry.unwrap_or("").trim()
    );
    if cache_key.trim().is_empty() {
        return String::new();
    }

    if let Some(cached) = icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&cache_key)
        .cloned()
    {
        return cached;
    }

    let icon = resolve_source_app_icon_data_url(source_app_id, desktop_entry).unwrap_or_default();
    icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key, icon.clone());
    icon
}

fn icon_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn resolve_source_app_icon_data_url(
    source_app_id: &str,
    desktop_entry: Option<&str>,
) -> Option<String> {
    let desktop_file = desktop_entry_candidates(source_app_id, desktop_entry)
        .into_iter()
        .find_map(find_desktop_file)?;
    let icon_name = read_desktop_entry_icon_name(&desktop_file)?;
    let icon_path = resolve_icon_path(&icon_name)?;
    image_path_to_data_url(&icon_path)
}

fn desktop_entry_candidates(source_app_id: &str, desktop_entry: Option<&str>) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    for value in [desktop_entry.unwrap_or_default(), source_app_id] {
        for candidate in normalize_desktop_entry_candidates(value) {
            if seen.insert(candidate.clone()) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

fn normalize_desktop_entry_candidates(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    push_candidate(&mut candidates, trimmed);

    if let Some(file_name) = Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
    {
        push_candidate(&mut candidates, file_name);
    }

    for separator in ['.', '-', '_'] {
        if let Some(tail) = trimmed.rsplit(separator).next() {
            push_candidate(&mut candidates, tail);
        }
    }

    candidates
}

fn push_candidate(candidates: &mut Vec<String>, value: &str) {
    let normalized = value.trim().trim_end_matches(".desktop").trim().to_string();
    if normalized.is_empty() {
        return;
    }

    if !candidates
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(&normalized))
    {
        candidates.push(normalized);
    }
}

fn find_desktop_file(entry: String) -> Option<PathBuf> {
    let path = Path::new(&entry);
    if path.is_absolute() && path.is_file() {
        return Some(path.to_path_buf());
    }

    let file_name = format!("{entry}.desktop");
    desktop_file_dirs()
        .into_iter()
        .find_map(|dir| find_file_case_insensitive(&dir, &file_name))
}

fn desktop_file_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        let data_home = PathBuf::from(data_home);
        dirs.push(data_home.join("applications"));
        dirs.push(data_home.join("flatpak/exports/share/applications"));
    } else if let Some(home) = env::var_os("HOME").filter(|value| !value.is_empty()) {
        let home = PathBuf::from(home);
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    let data_dirs = env::var_os("XDG_DATA_DIRS")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        });

    dirs.extend(data_dirs.into_iter().map(|dir| dir.join("applications")));
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));
    dirs
}

fn find_file_case_insensitive(dir: &Path, file_name: &str) -> Option<PathBuf> {
    let direct = dir.join(file_name);
    if direct.is_file() {
        return Some(direct);
    }

    let expected = file_name.to_ascii_lowercase();
    fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        (name == expected && entry.path().is_file()).then(|| entry.path())
    })
}

fn read_desktop_entry_icon_name(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_desktop_entry = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line.eq_ignore_ascii_case("[Desktop Entry]");
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "Icon" {
            let icon = value.trim().trim_matches('"').to_string();
            return (!icon.is_empty()).then_some(icon);
        }
    }

    None
}

fn resolve_icon_path(icon_name: &str) -> Option<PathBuf> {
    let icon_path = Path::new(icon_name);
    if icon_path.is_absolute() && icon_path.is_file() {
        return Some(icon_path.to_path_buf());
    }

    icon_theme_dirs()
        .into_iter()
        .find_map(|dir| find_icon_in_theme_dir(&dir, icon_name))
}

fn icon_theme_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        let data_home = PathBuf::from(data_home);
        dirs.push(data_home.join("icons"));
        dirs.push(data_home.join("flatpak/exports/share/icons"));
    } else if let Some(home) = env::var_os("HOME").filter(|value| !value.is_empty()) {
        let home = PathBuf::from(home);
        dirs.push(home.join(".local/share/icons"));
        dirs.push(home.join(".local/share/flatpak/exports/share/icons"));
    }

    if let Some(home) = env::var_os("HOME").filter(|value| !value.is_empty()) {
        dirs.push(PathBuf::from(home).join(".icons"));
    }

    let data_dirs = env::var_os("XDG_DATA_DIRS")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        });

    dirs.extend(data_dirs.into_iter().map(|dir| dir.join("icons")));
    dirs.push(PathBuf::from("/usr/share/pixmaps"));
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/icons"));
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/icons"));
    dirs
}

fn find_icon_in_theme_dir(dir: &Path, icon_name: &str) -> Option<PathBuf> {
    let mut best: Option<(PathBuf, u32)> = None;
    find_icon_in_dir_recursive(dir, icon_name, &mut best);
    best.map(|(path, _)| path)
}

fn find_icon_in_dir_recursive(dir: &Path, icon_name: &str, best: &mut Option<(PathBuf, u32)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            find_icon_in_dir_recursive(&path, icon_name, best);
            continue;
        }

        if !file_type.is_file() || !icon_file_matches(&path, icon_name) {
            continue;
        }

        let score = icon_path_score(&path);
        if best
            .as_ref()
            .map(|(_, current)| score > *current)
            .unwrap_or(true)
        {
            *best = Some((path, score));
        }
    }
}

fn icon_file_matches(path: &Path, icon_name: &str) -> bool {
    let expected_stem = icon_lookup_stem(icon_name);
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return false;
    };
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    if !["png", "svg", "jpg", "jpeg", "webp", "xpm"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    {
        return false;
    }

    stem.eq_ignore_ascii_case(&expected_stem)
}

fn icon_lookup_stem(icon_name: &str) -> String {
    Path::new(icon_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(icon_name)
        .trim()
        .to_string()
}

fn icon_path_score(path: &Path) -> u32 {
    let path_text = path.to_string_lossy().to_ascii_lowercase();
    let size_score = [512, 256, 192, 128, 96, 64, 48, 32, 24, 16]
        .into_iter()
        .find_map(|size| {
            path_text
                .contains(&format!("{size}x{size}"))
                .then_some(size)
        })
        .unwrap_or(1);
    let scalable_bonus = if path_text.contains("scalable") {
        512
    } else {
        0
    };
    scalable_bonus + size_score
}

fn image_path_to_data_url(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }

    let original_content_type = path
        .extension()
        .and_then(|value| value.to_str())
        .map(extension_to_content_type)
        .unwrap_or_else(|| detect_image_content_type(&bytes));

    if original_content_type != "image/svg+xml" && original_content_type != "image/x-xpixmap" {
        if let Some(data_url) = raster_image_to_png_data_url(&bytes) {
            return Some(data_url);
        }
    }

    Some(format!(
        "data:{original_content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    ))
}

fn raster_image_to_png_data_url(bytes: &[u8]) -> Option<String> {
    let image = image::load_from_memory(bytes).ok()?;
    let resized = image.resize_to_fill(ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
    let mut png = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .ok()?;
    if png.is_empty() {
        return None;
    }

    Some(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(png)
    ))
}

fn extension_to_content_type(extension: &str) -> String {
    match extension.to_ascii_lowercase().as_str() {
        "png" => "image/png".into(),
        "svg" => "image/svg+xml".into(),
        "jpg" | "jpeg" => "image/jpeg".into(),
        "webp" => "image/webp".into(),
        "xpm" => "image/x-xpixmap".into(),
        _ => "image/png".into(),
    }
}

fn detect_image_content_type(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg".into()
    } else if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        "image/png".into()
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp".into()
    } else if looks_like_svg(bytes) {
        "image/svg+xml".into()
    } else {
        "image/png".into()
    }
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let sample = bytes.len().min(512);
    let text = String::from_utf8_lossy(&bytes[..sample]).to_ascii_lowercase();
    text.contains("<svg")
}
