pub type DError = Box<dyn std::error::Error + Send + Sync>;
pub type DResult<T> = std::result::Result<T, DError>;
