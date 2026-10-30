//! Custom background picture. The chosen file is copied into the app data dir so it
//! survives the original being moved or deleted, and is handed to the UI as a data URL.

use std::{fs, path::PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};
use tauri::{AppHandle, Manager};

use crate::error::{Error, Result};

const STEM: &str = "background";
const TYPES: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("webp", "image/webp"),
    ("gif", "image/gif"),
    ("bmp", "image/bmp"),
];

fn data_dir(app: &AppHandle) -> Result<PathBuf> {
    app.path().app_data_dir().map_err(|e| Error::Other(e.to_string()))
}

fn mime_for(ext: &str) -> Option<&'static str> {
    TYPES.iter().find(|(e, _)| e.eq_ignore_ascii_case(ext)).map(|(_, m)| *m)
}

/// The stored background file, if any.
fn current(app: &AppHandle) -> Result<Option<(PathBuf, &'static str)>> {
    let dir = data_dir(app)?;
    for (ext, mime) in TYPES {
        let path = dir.join(format!("{STEM}.{ext}"));
        if path.exists() {
            return Ok(Some((path, mime)));
        }
    }
    Ok(None)
}

fn to_data_url(path: &PathBuf, mime: &str) -> Result<String> {
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(fs::read(path)?)))
}

pub fn get(app: &AppHandle) -> Result<Option<String>> {
    current(app)?.map(|(path, mime)| to_data_url(&path, mime)).transpose()
}

pub fn set(app: &AppHandle, source: &str) -> Result<String> {
    let source = PathBuf::from(source);
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| Error::Other("Unsupported image type".into()))?;
    let mime = mime_for(&ext).ok_or_else(|| Error::Other("Unsupported image type".into()))?;

    clear(app)?;
    let dir = data_dir(app)?;
    fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{STEM}.{ext}"));
    fs::copy(&source, &dest)?;
    to_data_url(&dest, mime)
}

pub fn clear(app: &AppHandle) -> Result<()> {
    while let Some((path, _)) = current(app)? {
        fs::remove_file(path)?;
    }
    Ok(())
}
