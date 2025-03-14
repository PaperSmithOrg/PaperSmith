// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dark_light::Mode;
use glob::glob;
use loader::write_project_config;
use log::info;
use log::warn;
use rfd::FileDialog;
use saving::create_directory;
use saving::create_empty_file;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use dark_light;

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
            write_project_config,
            create_directory,
            log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_systemtheme() -> String {
    let system_theme = dark_light::detect();

    match system_theme {
        Ok(theme) => {
            match theme {
                Mode::Dark => String::from("Dark"),
                Mode::Light => String::from("Light"),
                Mode::Unspecified => String::new(),
            }
        },
        Err(e) => {
            log(format!("Error getting System Theme: {e:?}"));
            String::new()
        }
    }
}

#[tauri::command]
fn log(msg: String) {
    let mut path = get_data_dir();
    path.push_str("/PaperSmith/papersmith.log");

    if !Path::new(path.as_str()).exists() {
        match File::create(path) {
            Ok(_) => return,
            Err(e) => {
                eprintln!("Error creating log file: {e:?}");
                return;
            }
        }
    }

    let mut file = match OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(path)
    {
        Ok(f) => {
            println!("created log file");   
            f
        },
        Err(e) => {
            eprintln!("Failed to open or create the file: {e:?}");
            return;
        }
    };

    let now = SystemTime::now();
    let log_msg = String::from(format!("[{now:?}]\t{msg:?}\n"));

    match file.write(log_msg.as_bytes()) {
        Ok(_) => return,
        Err(e) => {
            eprintln!("Failed to write to log file: {e:?}");
            return;
        }
    }
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
        #[allow(clippy::zombie_processes)]
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .expect("Failed to open directory in file explorer");
    }
}

#[tauri::command]
fn list_statistic_files() -> Result<Vec<String>, String> {
    let mut path = get_data_dir();
    path.push_str("/PaperSmith/Statistics/");

    let pattern = format!("{path}/**/*");
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
                                eprintln!("Unrecognized file name format: {original_name}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading entry: {e}");
                        return Err(format!("Error reading entry: {e}"));
                    }
                }
            }
        }
        Err(e) => return Err(format!("Invalid glob pattern: {e}")),
    }

    Ok(files)
}

fn format_file_name(file_name: &str) -> Option<String> {
    if let Some(stripped) = file_name.strip_suffix(".json") {
        let parts: Vec<&str> = stripped.split('T').collect();
        if parts.len() == 2 {
            let date = parts[0];
            let time = parts[1].replace('-', ":");
            return Some(format!("{date} {time}"));
        }
    }
    None
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn unformat_file_name(name: String) -> Option<String> {
    let parts: Vec<&str> = name.split(' ').collect();
    if parts.len() == 2 {
        let date = parts[0];
        let time = parts[1].replace(':', "-");
        return Some(format!("{date}T{time}.json"));
    }
    None
}

#[tauri::command]
fn read_json_file(path: String) -> Option<String> {
    fs::read_to_string(path).map_or(None, |content| {
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => Some(content),
            Err(_) => None,
        }
    })
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

// Definiere eine globale Variable für die Startzeit
// lazy_static! {
//     static ref START_TIME: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
// }

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn write_to_json(path: String, name: String, content: String) {
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
#[allow(clippy::needless_pass_by_value)]
fn get_settings(path: String) -> Settings {
    let file_path = format!("{path}/settings.json");

    let file = File::open(&file_path);

    match file {
        Ok(mut x) => {
            let mut content = String::new();
            let _ = x.read_to_string(&mut content);

            let json_content: Result<Settings, serde_json::Error> = serde_json::from_str(&content);

            match json_content {
                Ok(settings) => {
                    return settings;
                }
                Err(e) => {
                    eprint!("Settings were not readable! Error: {e:?}");
                }
            }

            Settings::default()
        }
        Err(e) => {
            eprint!("{e}");
            let mut settings = Settings::default();
            let system_theme = get_systemtheme();

            if system_theme != String::new() {
                settings.theme = system_theme;
            }

            settings
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
#[allow(clippy::needless_pass_by_value)]
fn write_to_file(path: String, content: String) {
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    // Ensure the directory exists
    let path = std::path::Path::new(&path);
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

    //println!("{:?}", content);

    // Write the content to the file
    match write!(file, "{content}") {
        Ok(()) => println!("Content appended to file: {path:?}"),
        Err(e) => eprintln!("Failed to write to file: {e:?}"),
    }
}
