use std::{io, net};

use crate::envelope;

pub struct Remote {
    pub stream: net::TcpStream,
    pub addr: net::SocketAddr,
}

impl Remote {
    pub fn new(stream: net::TcpStream, addr: net::SocketAddr) -> Self {
        Self {
            stream: stream,
            addr,
        }
    }

    pub fn next_envelope(&mut self) -> Result<envelope::Envelope, io::Error> {
        envelope::Envelope::read(&mut self.stream).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    pub fn wrap_and_send_envelope(&mut self, data: Vec<u8>) -> Result<(), io::Error> {
        let envelope = envelope::Envelope::new(data);

        envelope.write(&mut self.stream)
    }
}