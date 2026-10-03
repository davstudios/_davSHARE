use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Clone)]
pub struct Item { pub path: String, pub name: String, pub size: u64, pub is_dir: bool }

#[derive(Serialize)]
pub struct ActionResult { pub ok: bool, pub title: String, pub message: String, pub details: String }

#[derive(Deserialize)]
pub struct ActionOptions { pub destination: Option<String>, pub format: Option<String>, pub text: Option<String> }

#[tauri::command]
pub fn inspect_paths(paths: Vec<String>) -> Vec<Item> {
    paths.into_iter().map(|raw| {
        let path = Path::new(&raw);
        let metadata = fs::metadata(path).ok();
        Item { path: raw.clone(), name: path.file_name().and_then(|v| v.to_str()).unwrap_or(&raw).to_string(), size: metadata.as_ref().map(|v| v.len()).unwrap_or(0), is_dir: metadata.as_ref().map(|v| v.is_dir()).unwrap_or(false) }
    }).collect()
}

pub fn result(ok: bool, title: &str, message: &str, details: String) -> ActionResult { ActionResult { ok, title: title.into(), message: message.into(), details } }

