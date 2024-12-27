// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rfd::FileDialog;
use saving::create_empty_file;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
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
            show_save_dialog,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[tauri::command]
fn get_file_content(path: String) -> String {
    let pathbuf = PathBuf::from(path.clone());
    println!("{}", path.clone());
    if pathbuf.exists() & pathbuf.is_file() {
        fs::read_to_string(path).expect("Should have been able to read the file")
    } else {
        String::new()
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

// Definiere eine globale Variable für die Startzeit
// lazy_static! {
//     static ref START_TIME: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
// }

#[tauri::command]
fn write_to_json(path: &str, name: &str, content: &str) {
    // let start_time = *START_TIME.lock().unwrap();
    // let formatted_time = start_time.format("%Y-%m-%dT%H-%M-%S").to_string();
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
    // match result {
    //     Ok(_) => println!("Wrote in file: {:?}", file_path),
    //     Err(e) => eprintln!("Error when writing in file: {:?}", e),
    // }
}

#[tauri::command]
fn get_settings(path: &str, name: &str) -> Option<Settings> {
    let file_name = format!("{name}.json");
    let file_path = format!("{path}/{file_name}");

    let file = File::open(&file_path);

    match file {
        Ok(mut x) => {
            let mut content = String::new();
            let _ = x.read_to_string(&mut content);

            let json_content: Settings = serde_json::from_str(&content).expect("JSON was not well-formatted");
            
            Some(json_content)
        },
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
