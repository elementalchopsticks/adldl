use std::fs::File;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};

use anyhow::Result;

pub struct Dcc {
    pub filename: String,
    pub size: usize,
    stream: TcpStream,
}

impl Dcc {
    pub fn new(filename: String, address: &str, size: usize) -> Result<Self> {
        let stream = TcpStream::connect(address)?;

        Ok(Self {
            filename,
            size,
            stream,
        })
    }

    pub fn download(mut self) -> Result<()> {
        let mut file = File::create(&self.filename)?;

        let mut buf = [0; 4096];
        let mut progress: usize = 0;

        while progress < self.size {
            let count = self.stream.read(&mut buf)?;
            progress += count;
            file.write_all(&buf[..count])?;
        }

        self.stream.shutdown(Shutdown::Both)?;
        file.flush()?;

        Ok(())
    }
}
