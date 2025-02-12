use std::fs::{self, File};
use std::path::PathBuf;

use shared::Project;

use crate::loader::parse_project;

#[tauri::command]
pub fn create_project(path: String) -> Option<Project> {
    let mut path = PathBuf::from(path);

    let _ = fs::create_dir(&path);

    path.push("Chapters");
    let _ = fs::create_dir(&path);
    path.pop();

    path.push("Extras");
    let _ = fs::create_dir(&path);
    path.pop();

    path.push("Backups");
    let _ = fs::create_dir(&path);
    path.pop();

    path.push("Exports");
    let _ = fs::create_dir(&path);
    path.pop();

    path.push(".papersmith");
    path.set_extension("json");
    let _ = File::create(&path);
    path.pop();

    path.push("Note");
    path.set_extension("md");
    let _ = File::create(&path);
    path.pop();

    parse_project(path)
}

use std::io;

fn move_dir_recursive(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    if src == dst {
        return Ok(());
    }
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in src.read_dir()? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                move_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::rename(&src_path, &dst_path)?;
            }
        }
        fs::remove_dir_all(src)?;
    } else {
        fs::rename(src, dst)?;
    }
    Ok(())
}

#[tauri::command]
pub fn create_empty_file(path: String) {
    let path = PathBuf::from(path);
    println!("Creating: {}", path.to_str().unwrap());
    //if !can_create_path(path).is_empty(){
    //    println!("Path did not exist");
    //    return;
    //}
    let _ = File::create(path);
}

#[tauri::command]
pub fn create_directory(path: String) {
    let path = PathBuf::from(path);
    println!("Creating Directory: {}", path.to_str().unwrap());
    //if !can_create_path(path).is_empty(){
    //    println!("Path did not exist");
    //    return;
    //}
    let _ = fs::create_dir(path);
}

#[tauri::command]
pub fn delete_path(path: String) {
    let path = PathBuf::from(path);
    println!("Deleting: {}", path.to_str().unwrap());
    if !path.exists() {
        println!("Path did not exist");
        return;
    }
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else {
        let _ = fs::remove_file(path);
    }
}

#[tauri::command]
pub fn rename_path(path: &str, old: &str, new: &str) {
    let old_path = PathBuf::from(&path).join(old);
    let new_path = PathBuf::from(&path).join(new);

    println!("Old path: {}", old_path.display());
    println!("New path: {}", new_path.display());

    if old_path.exists() {
        move_dir_recursive(&old_path, &new_path).unwrap();
    } else {
        println!("Old path does not exist.");
    }
}

#[tauri::command]
pub fn add_chapter(path: String) {
    let mut path = PathBuf::from(path);
    let _ = fs::create_dir(&path);

    path.push("Note");
    path.set_extension("md");
    let _ = File::create(&path);
    path.pop();

    path.push("Content");
    path.set_extension("md");
    let _ = File::create(&path);
    path.pop();
}
