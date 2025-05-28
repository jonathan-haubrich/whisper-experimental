use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Client connected")]
    ClientNotConnected,

    #[error("Client is already connected")]
    ClientAlreadyConnected,
}