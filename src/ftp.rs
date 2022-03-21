use std::io::prelude::*;
use std::net::TcpStream;
use std::io::BufReader;

pub enum ConnectionType {
    Passive,
    Active
}

pub struct FTPConnection {
    control_stream: TcpStream,
    data_stream: Option<TcpStream>,
    r#type: ConnectionType
}

impl FTPConnection {
    pub fn new(hostname: String, connection_type: ConnectionType) -> std::io::Result<FTPConnection> {
        Ok(FTPConnection { control_stream: TcpStream::connect([hostname, String::from(":21")].concat())?, data_stream: None, r#type: connection_type })
    }
    pub fn write_to_control_stream(&mut self, input: String) -> std::io::Result<()> {
        self.control_stream.write(&input.as_bytes())?;
        Ok(())
    }
    pub fn read_server_response(&mut self) -> std::io::Result<String> {
        let mut res = String::new();
        let mut reader = BufReader::new(&self.control_stream);
        reader.read_line(&mut res);
        Ok(res)
    }
}
