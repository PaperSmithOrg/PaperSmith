use serde::{Deserialize, Serialize};
use std::fmt::{self};
use std::path::PathBuf;


#[derive(Serialize, Deserialize)]
pub struct FileWriteData {
    pub path: String,
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Settings {
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: String::from("Light"),
        }
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Settings:")?;
        writeln!(f, "Theme: {:?}", self.theme)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Project {
    pub path: PathBuf,
    pub chapters: Vec<Chapter>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Chapter {
    pub name: String,
    pub notes: Vec<String>,
    pub extras: Vec<String>,
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Project Path: {:?}", self.path)?;
        writeln!(f, "Chapters:")?;
        for (index, chapter) in self.chapters.iter().enumerate() {
            writeln!(f, "  Chapter {}: {}", index + 1, chapter)?;
        }
        Ok(())
    }
}

impl fmt::Display for Chapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "  Notes:")?;
        for (index, note) in self.notes.iter().enumerate() {
            writeln!(f, "    {}: {}", index + 1, note)?;
        }
        writeln!(f, "  Extras:")?;
        for (index, extra) in self.extras.iter().enumerate() {
            writeln!(f, "    {}: {}", index + 1, extra)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaperSmithError {
    code: usize,
    message: Option<String>,
}

impl PaperSmithError {
    pub const fn new(code: usize, message: String) -> Self {
        Self {
            code,
            message: Some(message),
        }
    }

    pub const fn new_only_code(code: usize) -> Self {
        Self {
            code,
            message: None,
        }
    }

    pub const fn code(&self) -> usize {
        self.code
    }

    pub const fn message(&self) -> Option<&String> {
        self.message.as_ref()
    }
}

impl fmt::Display for PaperSmithError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.code {
            404 => "Sorry, Can not find the Page!",
            2 => "Not a valid Project",
            _ => "Sorry, something is wrong! Please Try Again!",
        };

        write!(f, "{err_msg}")
    }
}
