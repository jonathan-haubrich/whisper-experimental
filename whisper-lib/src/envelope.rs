use std::net;
use std::io::{Read, Write};

use prost::{self, Message};

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

        let len_ptr = &mut envelope.len as *mut _ as *mut u8;
        let len_size = size_of_val(&envelope.len);

        let mut len_slice = unsafe { std::slice::from_raw_parts_mut(len_ptr, len_size) };

        stream.read_exact(&mut len_slice)?;

        envelope.len = envelope.len.to_le();

        envelope.data.resize(envelope.len as usize, 0);

        stream.read_exact(&mut envelope.data)?;

        Ok(envelope)
    }

    pub fn write(&self, stream: &mut net::TcpStream) -> Result<(), std::io::Error> {
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