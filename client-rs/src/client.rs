use std::{io::Write, net::{TcpStream, ToSocketAddrs}};
use anyhow::Result;

use crate::error::ClientError;

use whisper_lib::{envelope, protocol};

pub struct Client {
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            stream: None,
        }
    }

    pub fn connect<A: ToSocketAddrs>(&mut self, endpoint: A) -> Result<()> {
        if self.stream.is_some() {
            return Err(ClientError::ClientAlreadyConnected.into());
        }
        
        let stream = TcpStream::connect(endpoint)?;

        self.stream = Some(stream);

        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }

    pub fn load_module(&mut self, data: Vec<u8>) -> Result<()> {
        let Some(mut stream) = self.stream.as_mut() else {
            return Err(ClientError::ClientNotConnected.into());
        };

        let tx_id = rand::random();
        let request = protocol::Request {
            header: Some(protocol::Header { tx_id }),
            request: Some(
                protocol::request::Request::Load(
                    protocol::LoadRequest {
                        data
                    }
                )
            )
        };

        let envelope: envelope::Envelope = request.into();

        envelope.write(&mut stream).map_err(|err| err.into())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.disconnect();
    }
}