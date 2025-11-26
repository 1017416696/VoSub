mod srt_parser;

use srt_parser::{read_srt_file, write_srt_file, SRTFile, SubtitleEntry};
use std::fs;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Backend was called with an argument: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Read and parse an SRT file
#[tauri::command]
fn read_srt(file_path: String) -> Result<SRTFile, String> {
    read_srt_file(&file_path)
}

/// Write SRT file
#[tauri::command]
fn write_srt(file_path: String, entries: Vec<SubtitleEntry>) -> Result<(), String> {
    write_srt_file(&file_path, &entries)
}

/// Read audio file and return as base64
#[tauri::command]
fn read_audio_file(file_path: String) -> Result<String, String> {
    let file_data = fs::read(&file_path)
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    // Convert to base64
    let base64_data = base64_encode(&file_data);
    Ok(base64_data)
}

/// Simple base64 encoding function
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b3 = if chunk.len() > 2 { chunk[2] } else { 0 };

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        result.push(CHARSET[((n >> 18) & 63) as usize] as char);
        result.push(CHARSET[((n >> 12) & 63) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[((n >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                let window = tauri::Manager::get_webview_window(_app, "main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, read_srt, write_srt, read_audio_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
