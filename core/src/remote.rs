use std::{io, net};

use binrw::{BinRead, BinResult, BinWrite};
use prost::Message;

use crate::Envelope;

pub struct Remote {
    pub stream: binrw::io::NoSeek<net::TcpStream>,
    pub addr: net::SocketAddr,
}

impl Remote {
    pub fn new(stream: net::TcpStream, addr: net::SocketAddr) -> Self {
        Self {
            stream: binrw::io::NoSeek::new(stream),
            addr,
        }
    }

    pub fn next_envelope(&mut self) -> Result<Envelope, io::Error> {
        crate::Envelope::read(&mut self.stream).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    pub fn wrap_and_send_envelope(&mut self, data: Vec<u8>) -> BinResult<()> {
        let envelope = Envelope::new(data);

        envelope.write(&mut self.stream)
    }
}