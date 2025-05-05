use std::{
    io::{self, Error},
    net,
};

use crate::remote::Remote;

pub struct Server {
    pub listener: Option<net::TcpListener>
}

impl Server {
    pub fn new() -> Self {
        Self {
            listener: None
        }
    }

    pub fn bind<A: net::ToSocketAddrs>(&mut self, endpoint: A) -> Result<(), io::Error>  {
        if let None = self.listener {
            _ = self.listener.insert(net::TcpListener::bind(endpoint)?);
            return Ok(());
        }

        Err(Error::new(io::ErrorKind::Other, "listener already bound"))
    }

    pub fn accept(&mut self) -> Result<Remote, io::Error> {
        if let Some(listener) = &self.listener {
            return match listener.accept() {
                Ok((stream, addr)) => Ok(Remote::new(stream, addr)),
                Err(err) => Err(err),
            }
        }

        Err(Error::new(io::ErrorKind::Other, "listener not bound"))
    }
}