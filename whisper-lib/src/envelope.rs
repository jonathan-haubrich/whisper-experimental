use std::net;
use std::io::{Read, Write};

use prost::{self, Message};

use log::{error, info, warn};

#[repr(C)]
pub struct Envelope {
    pub len: u64,
    pub data: Vec<u8>,
}

impl Envelope {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            len: data.len() as u64,
            data
        }
    }

    pub fn read(stream: &mut net::TcpStream) -> Result<Self, std::io::Error> {
        let mut envelope = Self {
            len: 0,
            data: Vec::new()
        };

        let mut len_slice = [0u8; 8];

        stream.read_exact(&mut len_slice)?;

        info!("read bytes: {:#?}", len_slice);

        info!("before to_le: envelope.len: {}", envelope.len);
        envelope.len = u64::from_ne_bytes(len_slice);
        info!("after to_le: envelope.len: {}", envelope.len);

        envelope.data.resize(envelope.len as usize, 0);

        stream.read_exact(&mut envelope.data)?;

        Ok(envelope)
    }

    pub fn write(&self, stream: &mut net::TcpStream) -> Result<(), std::io::Error> {
        info!("ne bytes: {:#?}", self.len.to_ne_bytes().as_slice());
        stream.write_all(self.len.to_ne_bytes().as_slice())?;
        stream.write_all(&self.data)
    }
}

impl<M: Message> From<M> for Envelope {    
    fn from(value: M) -> Self {
        let data = value.encode_to_vec();
        
        let envelope = Envelope::new(data);

        envelope
    }
}