use std::net::{TcpStream, ToSocketAddrs};
use anyhow::Result;
use log::info;

use crate::error::ClientError;

use whisper_lib::{envelope, protocol::{self, request, Header}};

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

    fn next_envelope(&mut self) -> Result<envelope::Envelope> {
        let Some(stream) = self.stream.as_mut() else {
            return Err(ClientError::ClientNotConnected.into());
        };

        envelope::Envelope::read(stream).map_err(|err| err.into())
    }

    pub fn next_message(&mut self) -> Result<protocol::Response> {
        let envelope = self.next_envelope()?;

        envelope.try_into().map_err(|err: std::io::Error| err.into())
    }

    pub fn load_module(&mut self, data: Vec<u8>) -> Result<protocol::Response> {
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

        info!("calling envelope.write");

        envelope.write(&mut stream)?;

        self.next_message()
    }

    pub fn send_command(&mut self, module_id: u64, command_id: u64, tx_id: u64, data: Option<Vec<u8>>) -> Result<protocol::Response> {
        let Some(mut stream) = self.stream.as_mut() else {
            return Err(ClientError::ClientNotConnected.into());
        };
        
        let request = protocol::Request {
            header: Some(protocol::Header{ tx_id }),
            request: Some(
                protocol::request::Request::Command(
                    protocol::CommandRequest {
                        id: command_id,
                        module_id: module_id,
                        data: data.unwrap_or_default(),
                    },
                ),
            ),
        };

        let envelope: envelope::Envelope = request.into();

        envelope.write(&mut stream)?;

        self.next_message()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.disconnect();
    }
}