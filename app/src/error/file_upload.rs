use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileUploadError {
    #[error("Multipart error：{0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),

    #[error("File size exceeds the limit: {0} bytes")]
    FileSizeExceeded(usize),

    #[error("File type not allowed: {0}")]
    FileTypeNotAllowed(String),

    #[error("File upload failed: {0}")]
    UploadFailed(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}
