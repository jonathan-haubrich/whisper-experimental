use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryModuleError {
    #[error("invalid pe data")]
    PeInvalid,
}

pub type Result<T> = std::result::Result<T, MemoryModuleError>;