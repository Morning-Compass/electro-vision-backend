use base64;
use infer;
use mime_guess;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::constants::MAX_MULTIMEDIA_SIZE; // To generate unique filenames

#[derive(Debug)]
pub enum MultimediaHandlerError {
    DecodingError,
    InvalidFileType,
    FileSystemError,
    MaximumFileSizeReached,
}

impl std::fmt::Display for MultimediaHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultimediaHandlerError::DecodingError => write!(f, "Decoding error"),
            MultimediaHandlerError::InvalidFileType => write!(f, "Invalid file type"),
            MultimediaHandlerError::FileSystemError => write!(f, "File system error"),
            MultimediaHandlerError::MaximumFileSizeReached => {
                write!(f, "Maximum file size reached")
            }
        }
    }
}

pub struct MultimediaHandler {
    content_base64: String, // Store the base64 string
    workspace_id: i32,
    base_dir: String,
    max_file_size_bytes: usize, // e.g., 5 * 1024 * 1024 for 5MB
}

impl MultimediaHandler {
    pub fn new(content_base64: String, workspace_id: i32) -> Self {
        MultimediaHandler {
            content_base64,
            workspace_id,
            base_dir: "user_multimedia".to_string(), // Or get from config
            max_file_size_bytes: MAX_MULTIMEDIA_SIZE as usize, // 5 MB default limit
        }
    }

    // Helper function to extract base64 part
    fn extract_base64_data(&self) -> Result<&str, MultimediaHandlerError> {
        if let Some(comma_index) = self.content_base64.find(',') {
            Ok(&self.content_base64[comma_index + 1..])
        } else {
            // If no comma is found, assume it's just the base64 data
            Ok(&self.content_base64)
        }
    }

    pub fn decode_and_store(&mut self) -> Result<PathBuf, MultimediaHandlerError> {
        let base64_data_slice = self.extract_base64_data()?;

        let bytes = base64::decode(base64_data_slice).map_err(|e| {
            eprintln!("decoding error: {:?}", e); // Log the specific base64 error
            MultimediaHandlerError::DecodingError
        })?;

        if bytes.len() > self.max_file_size_bytes {
            return Err(MultimediaHandlerError::MaximumFileSizeReached);
        }

        let inferred_type = infer::get(&bytes).ok_or(MultimediaHandlerError::InvalidFileType)?;
        let extension = inferred_type.extension();

        let mime_type_guesser = mime_guess::MimeGuess::from_ext(extension);
        let mime_type = mime_type_guesser.first_or_octet_stream();

        let subdir = match mime_type.type_().as_str() {
            "image" => "images",
            "video" => "videos",
            _ => return Err(MultimediaHandlerError::InvalidFileType),
        };

        let workspace_path = Path::new(&self.base_dir).join(self.workspace_id.to_string());
        let final_dir = workspace_path.join(subdir);

        // Create directories if they don't exist
        fs::create_dir_all(&final_dir).map_err(|e| {
            eprintln!("filesystem error creating dir: {:?}", e);
            MultimediaHandlerError::FileSystemError
        })?;

        let filename = format!("{}.{}", Uuid::new_v4(), extension);
        let file_path = final_dir.join(filename);

        fs::write(&file_path, &bytes).map_err(|e| {
            eprintln!("filesystem error writing file: {:?}", e);
            MultimediaHandlerError::FileSystemError
        })?;

        Ok(file_path)
    }

    pub fn get_file_content_base64(file_path: &str) -> Result<String, MultimediaHandlerError> {
        let path = PathBuf::from(file_path);
        let bytes = fs::read(&path).map_err(|e| {
            eprintln!("Error reading file for base64 encoding: {:?}", e);
            MultimediaHandlerError::FileSystemError
        })?;

        let mime_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();

        let base64_data = base64::encode(&bytes);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    pub fn remove_file_by_path(file_path: &str) -> Result<(), MultimediaHandlerError> {
        let path = PathBuf::from(file_path);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                eprintln!("Error removing file: {:?}", e);
                MultimediaHandlerError::FileSystemError
            })?;
        }
        Ok(())
    }
}
