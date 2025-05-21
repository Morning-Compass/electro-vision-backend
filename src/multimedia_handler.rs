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

        let mime_type = MimeGuess::from_ext(
            infer::get(&bytes)
                .map(|info| info.extension())
                .unwrap_or("bin"),
        )
        .first_or_octet_stream();

        let user_dir = format!("user_multimedia/{}", self.workspace_id);
        let media_type_dir = match mime_type.type_().as_str() {
            "image" => "images",
            "video" => "videos",
            _ => return Err(MultimediaHandlerError::InvalidFileType),
        };
        let full_dir = format!("{}/{}", user_dir, media_type_dir);

        let mut file_created = false;
        let mut created_path: Option<PathBuf> = None;

        if let Err(e) = fs::create_dir_all(&full_dir) {
            eprintln!("filesys err: {:?}", e);
            return Err(MultimediaHandlerError::FileSystemError);
        }

        let extension = mime_type.subtype().to_string();
        let file_name = format!("{}.{}", Uuid::new_v4(), extension);
        let file_path = Path::new(&full_dir).join(&file_name);

        match fs::File::create(&file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&bytes) {
                    eprintln!("write error: {:?}", e);
                    fs::remove_file(&file_path);
                    fs::remove_dir_all(&user_dir);
                    return Err(MultimediaHandlerError::FileSystemError);
                } else {
                    file_created = true;
                    created_path = Some(file_path.clone());
                }
            }
            Err(e) => {
                eprintln!("create error: {:?}", e);
                let _ = fs::remove_dir_all(&user_dir);
                return Err(MultimediaHandlerError::FileSystemError);
            }
        }

        self.file_name = Some(file_name);
        Ok(created_path.unwrap())
    }

    pub fn get_file_path(&self) -> Option<String> {
        self.file_name.as_ref().map(|name| {
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

    pub fn edit_multimedia_from_path(
        base64_content: String,
        file_path_str: &str,
    ) -> Result<(), MultimediaHandlerError> {
        let bytes = general_purpose::STANDARD
            .decode(&base64_content)
            .map_err(|_| MultimediaHandlerError::DecodingError)?;

        if bytes.len() > MAX_MULTIMEDIA_SIZE as usize {
            return Err(MultimediaHandlerError::MaximumFileSizeReached);
        }

        let file_path = Path::new(file_path_str);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|_| MultimediaHandlerError::FileSystemError)?;
        }

        let mut file =
            fs::File::create(file_path).map_err(|_| MultimediaHandlerError::FileSystemError)?;

        file.write_all(&bytes)
            .map_err(|_| MultimediaHandlerError::FileSystemError)?;

        Ok(())
    }
    pub fn get_file_content_base64(file_path: &str) -> Result<String, io::Error> {
        let bytes = fs::read(file_path)?;
        Ok(general_purpose::STANDARD.encode(&bytes))
    }

    pub fn remove_file_by_path(file_path_str: &str) -> Result<(), MultimediaHandlerError> {
        let file_path = Path::new(file_path_str);

        if file_path.exists() {
            fs::remove_file(file_path).map_err(|_| MultimediaHandlerError::FileSystemError)?;
        }

        Ok(())
    }
}
