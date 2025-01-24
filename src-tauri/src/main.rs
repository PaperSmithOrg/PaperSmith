// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use glob::glob;
use loader::write_project_config;
use log::info;
use log::warn;
use rfd::FileDialog;
use saving::create_empty_file;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;

mod loader;
use loader::parse_project;

mod checking;
use checking::can_create_path;
use checking::choose_folder;

mod saving;
use saving::add_chapter;
use saving::create_project;
use saving::delete_path;
use saving::rename_path;

use shared::Project;
use shared::Settings;

fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_project,
            write_to_file,
            write_to_json,
            choose_folder,
            can_create_path,
            create_project,
            get_data_dir,
            get_documents_folder,
            rename_path,
            add_chapter,
            delete_path,
            open_explorer,
            create_empty_file,
            get_file_content,
            get_settings,
            list_statistic_files,
            unformat_file_name,
            read_json_file,
            write_project_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[tauri::command]
fn get_file_content(path: String) -> String {
    info!("Reading file: {path}");
    match fs::read_to_string(path) {
        Ok(string) => string,
        Err(e) => {
            warn!("Error reading file: {e}");
            String::new()
        }
    }
}

#[tauri::command]
fn open_explorer(path: String) {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path.clone())
            .spawn()
            .expect("Failed to open directory in Explorer");
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .expect("Failed to open directory in file explorer");
    }
}

#[tauri::command]
fn list_statistic_files() -> Result<Vec<String>, String> {
    let mut path = get_data_dir();
    path.push_str("/PaperSmith/");

    let pattern = format!("{}/**/*", path);
    let mut files = Vec::new();

    match glob(&pattern) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(path) => {
                        if let Some(file_name) = path.file_name() {
                            let original_name = file_name.to_string_lossy();
                            if let Some(formatted_name) = format_file_name(&original_name) {
                                files.push(formatted_name);
                            } else {
                                eprintln!("Unrecognized file name format: {}", original_name);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading entry: {}", e);
                        return Err(format!("Error reading entry: {}", e));
                    }
                }
            }
        }
        Err(e) => return Err(format!("Invalid glob pattern: {}", e)),
    }

    Ok(files)
}

fn format_file_name(file_name: &str) -> Option<String> {
    if let Some(stripped) = file_name.strip_suffix(".json") {
        let parts: Vec<&str> = stripped.split('T').collect();
        if parts.len() == 2 {
            let date = parts[0];
            let time = parts[1].replace("-", ":");
            return Some(format!("{} {}", date, time));
        }
    }
    None
}

#[tauri::command]
fn unformat_file_name(name: String) -> Option<String> { 
    let parts: Vec<&str> = name.split(' ').collect();
    if parts.len() == 2 {
        let date = parts[0];
        let time = parts[1].replace(":", "-");
        return Some(format!("{}T{}.json", date, time));
    }
    None
}

#[tauri::command]
fn read_json_file(path: String) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => Some(content),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

#[tauri::command]
fn get_project() -> Option<Project> {
    FileDialog::new().pick_folder().and_then(parse_project)
}

#[tauri::command]
fn get_documents_folder() -> String {
    dirs_next::document_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

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
// lazy_static! {
//     static ref START_TIME: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
// }

#[tauri::command]
fn write_to_json(path: &str, name: &str, content: &str) {
    let file_name = format!("{name}.json");
    let file_path = format!("{path}/{file_name}");

    let mut file = match File::create(&file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error when creating: {e:?}");
            return;
        }
    };

    let _result = write!(file, "{content}");
}

#[tauri::command]
fn get_settings(path: &str) -> Option<Settings> {
    let file_path = format!("{path}/settings.json");

    let file = File::open(&file_path);

    match file {
        Ok(mut x) => {
            let mut content = String::new();
            let _ = x.read_to_string(&mut content);

            let json_content: Settings =
                serde_json::from_str(&content).expect("JSON was not well-formatted");

            println!("{}", json_content.theme);

            Some(json_content)
        }
        Err(e) => {
            eprint!("{e}");
            let settings = Settings::default();

            Some(settings)
        }
    }
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

    let mut file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open or create the file: {e:?}");
            return;
        }
    };

    println!("{:?}", content);

    // Write the content to the file
    match write!(file, "{content}") {
        Ok(()) => println!("Content appended to file: {path:?}"),
        Err(e) => eprintln!("Failed to write to file: {e:?}"),
    }
}
