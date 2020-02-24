use async_std::{net::TcpStream, prelude::*};
use std::io;
pub mod format;
use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

const HEADER_SIZE: usize = 4;

#[derive(Error, Debug)]
pub enum MotionError {
    #[error("IO related error")]
    IOError(#[from] io::Error),
}

#[derive(Default)]
pub struct Client {
    stream: Option<TcpStream>,
    description: Option<String>,
}

impl Client {
    pub async fn connect(&mut self, host: &str, port: i32) -> Result<(), MotionError> {
        let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
        self.stream = Some(stream);
        let desc_msg = self.receive().await?;
        self.description = desc_msg.map(|val| String::from_utf8(val).ok()).flatten();
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<Vec<u8>>, MotionError> {
        if let Some(header) = self.read_header().await? {
            let msg = self.read_rawdata(header).await?;
            Ok(msg)
        } else {
            Ok(None)
        }
    }

    async fn read_header(&mut self) -> Result<Option<i32>, MotionError> {
        if let Some(stream) = &mut self.stream {
            let mut buf = [0_u8; HEADER_SIZE];
            stream.read(&mut buf).await?;
            let length = i32::from_be_bytes(buf);
            Ok(Some(length))
        } else {
            Ok(None)
        }
    }

    async fn read_rawdata(&mut self, length: i32) -> Result<Option<Vec<u8>>, MotionError> {
        if let Some(stream) = &mut self.stream {
            let mut buf = vec![0_u8; length as usize];
            stream.read_exact(&mut buf).await?;
            Ok(Some(buf))
        } else {
            Ok(None)
        }
    }

    pub async fn write(&mut self, data: Vec<u8>) -> Result<(), MotionError> {
        if let Some(stream) = &mut self.stream {
            let header = (data.len() as i32).to_be_bytes();
            let msg = [&header[..], &data[..]].concat();
            stream.write_all(&msg).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RootNode {
    id: String,
    key: i32,
    tracking: Option<i32>,
    #[serde(rename = "node", default)]
    pub nodes: Option<Vec<Node>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Node {
    pub id: String,
    pub key: i32,
    pub active: Option<i32>,
}

#[cfg(test)]
mod tests {}
