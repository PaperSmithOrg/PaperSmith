// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use rfd::FileDialog;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

mod loader;
use loader::parse_project;

mod checking;
use checking::can_create_path;
use checking::check_if_folder_exists;
use checking::choose_folder;

mod menu;
use menu::generate as generate_menu;

mod saving;
use saving::create_project;
use saving::rename_path;

use shared::Project;

fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            show_save_dialog,
            extract_div_contents,
            get_project,
            write_to_file,
            write_to_json,
            choose_folder,
            check_if_folder_exists,
            can_create_path,
            create_project,
            get_data_dir,
            get_documents_folder,
            rename_path
        ])
        .menu(generate_menu())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn show_save_dialog() -> Result<String, String> {
    let path = FileDialog::new()
        .set_title("Save File")
        .add_filter("Text", &["txt"])
        .add_filter("MarkDown", &["md"])
        .save_file()
        .ok_or_else(|| "No file selected".to_string())?;

    Ok(path.to_str().unwrap_or_default().to_string())
}

#[tauri::command]
fn get_project() -> Option<Project> {
    let project_path = FileDialog::new().pick_folder().unwrap();
    parse_project(project_path)
}

#[tauri::command]
fn get_documents_folder() -> String {
    dirs_next::document_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
/*this one worked--------------------------------------------------------------
#[tauri::command]
async fn show_save_dialog() {
    let test: &str = "Test";
    println!("{}", test);
    dialog::FileDialogBuilder::default()
        .add_filter("Markdown", &["md"])
        .pick_file(|path_buf| match path_buf {
            Some(p) => {}
            _ => {}
        });
}*/

#[tauri::command]
fn extract_div_contents(input: &str) -> Vec<String> {
    // Initialize an empty vector to store the extracted contents
    let mut result = Vec::new();

    // Define the start and end tag strings
    let start_tag = "<div>";
    let end_tag = "</div>";

    // Split the input string by the start tag
    let parts: Vec<&str> = input.split(start_tag).collect();

    // Iterate over the parts and extract the contents between the start and end tags
    for part in parts {
        if let Some(end_index) = part.find(end_tag) {
            if part.contains("<br>") {
            } else {
                let content = &part[..end_index];
                result.push(content.to_string());
            }
        }
    }
    result
}

// Definiere eine globale Variable für die Startzeit
lazy_static! {
    static ref START_TIME: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
}

#[tauri::command]
fn write_to_json(path: &str, content: &str) {
    let start_time = *START_TIME.lock().unwrap();
    let formatted_time = start_time.format("%Y-%m-%dT%H-%M-%S").to_string();
    let file_name = format!("{formatted_time}.json");
    let file_path = format!("{path}/{file_name}");

    let mut file = match File::create(&file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error when creating: {e:?}");
            return;
        }
    };
    let _result = write!(file, "{content}");
    // match result {
    //     Ok(_) => println!("Wrote in file: {:?}", file_path),
    //     Err(e) => eprintln!("Error when writing in file: {:?}", e),
    // }
}

#[tauri::command]
fn get_data_dir() -> String {
    if let Some(config_dir) = dirs_next::data_dir() {
        return config_dir.to_string_lossy().to_string();
    }
    "No path".to_string()
}

#[tauri::command]
fn write_to_file(path: &str, content: &str) {
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    // Ensure the directory exists
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            match fs::create_dir_all(parent) {
                Ok(()) => println!("Directory created: {parent:?}"),
                Err(e) => eprintln!("Failed to create directory: {e:?}"),
            }
        }
    }

    // Open the file in append mode or create it if it doesn't exist
    let mut file = match OpenOptions::new().append(true).create(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open or create the file: {e:?}");
            return;
        }
    };

    // Write the content to the file
    match write!(file, "{content}") {
        Ok(()) => println!("Content appended to file: {path:?}"),
        Err(e) => eprintln!("Failed to write to file: {e:?}"),
    }
}
