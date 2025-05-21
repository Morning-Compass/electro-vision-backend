use base64::{engine::general_purpose, Engine as _};
use mime_guess::MimeGuess;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};
use uuid::Uuid;

use crate::constants::MAX_MULTIMEDIA_SIZE;

#[derive(Debug)]
pub enum MultimediaHandlerError {
    MaximumFileSizeReached,
    DecodingError,
    FileSystemError,
    InvalidFileType,
}

pub struct MultimediaHandler {
    pub multimedia: String,
    pub workspace_id: i32,
    pub file_name: Option<String>,
}

impl MultimediaHandler {
    pub fn new(multimedia: String, workspace_id: i32) -> Self {
        Self {
            multimedia,
            workspace_id,
            file_name: None,
        }
    }

    // This function will now be responsible for both initial creation and "re-creation"
    // when the file type changes. It returns the newly created PathBuf.
    pub fn decode_and_store(&mut self) -> Result<PathBuf, MultimediaHandlerError> {
        let bytes = general_purpose::STANDARD
            .decode(&self.multimedia)
            .map_err(|e| {
                eprintln!("decoding error: {:?}", e);
                MultimediaHandlerError::DecodingError
            })?;

        if bytes.len() > MAX_MULTIMEDIA_SIZE as usize {
            return Err(MultimediaHandlerError::MaximumFileSizeReached);
        }

        let inferred_type = infer::get(&bytes).ok_or(MultimediaHandlerError::InvalidFileType)?;
        let mime_type = MimeGuess::from_ext(inferred_type.extension()).first_or_octet_stream();

        let user_dir = format!("user_multimedia/{}", self.workspace_id);
        let media_type_dir = match mime_type.type_().as_str() {
            "image" => "images",
            "video" => "videos",
            _ => return Err(MultimediaHandlerError::InvalidFileType),
        };
        let full_dir = format!("{}/{}", user_dir, media_type_dir);

        if let Err(e) = fs::create_dir_all(&full_dir) {
            eprintln!("filesys err: {:?}", e);
            return Err(MultimediaHandlerError::FileSystemError);
        }

        let extension = inferred_type.extension().to_string();
        let file_name = format!("{}.{}", Uuid::new_v4(), extension);
        let file_path = Path::new(&full_dir).join(&file_name);

        match fs::File::create(&file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&bytes) {
                    eprintln!("write error: {:?}", e);
                    let _ = fs::remove_file(&file_path); // Clean up partially written file
                    return Err(MultimediaHandlerError::FileSystemError);
                } else {
                    self.file_name = Some(file_name); // Store the newly generated file name
                    Ok(file_path) // Return the full path to the new file
                }
            }
            Err(e) => {
                eprintln!("create error: {:?}", e);
                return Err(MultimediaHandlerError::FileSystemError);
            }
        }
    }

    // This method is now redundant as `decode_and_store` handles new file creation.
    // If you need to retrieve the path of a file that was just stored, you can use the
    // PathBuf returned by `decode_and_store`.
    pub fn get_file_path(&self) -> Option<String> {
        self.file_name.as_ref().map(|name| {
            // Simplified logic as `decode_and_store` already ensures correct directory.
            // This is primarily for getting the path of a file that has been stored by THIS handler instance.
            let media_type = if name.ends_with(".mp4") || name.ends_with(".mov") {
                "videos"
            } else {
                "images"
            };
            format!(
                "user_multimedia/{}/{}/{}",
                self.workspace_id, media_type, name
            )
        })
    }

    pub fn remove_user_data(&self) -> Result<(), MultimediaHandlerError> {
        let user_dir = format!("user_multimedia/{}", self.workspace_id);
        if Path::new(&user_dir).exists() {
            fs::remove_dir_all(&user_dir).map_err(|e| {
                eprintln!("failed to remove user dir: {:?}", e);
                MultimediaHandlerError::FileSystemError
            })?;
        }
        Ok(())
    }

    // This function is no longer needed.
    // When a multimedia item is updated, the old one should be removed and a new one created.
    // pub fn edit_multimedia_from_path(...) { ... }

    pub fn get_file_content_base64(file_path: &str) -> Result<String, io::Error> {
        let bytes = fs::read(file_path)?;
        Ok(general_purpose::STANDARD.encode(&bytes))
    }

    pub fn remove_file_by_path(file_path_str: &str) -> Result<(), MultimediaHandlerError> {
        let file_path = Path::new(file_path_str);

        if file_path.exists() {
            fs::remove_file(file_path).map_err(|e| {
                eprintln!("failed to remove file {}: {:?}", file_path_str, e);
                MultimediaHandlerError::FileSystemError
            })?;
        }

        Ok(())
    }
}
