extern crate snafu;

use snafu::prelude::*;
use std::io::prelude::*;
use std::net::TcpStream;
use std::io::BufReader;
use std::io::ErrorKind;

pub enum ConnectionType {
    Passive,
    Active
}

pub struct FTPConnection {
    control_stream: TcpStream,
    data_stream: Option<TcpStream>,
    r#type: ConnectionType
}

#[derive(Debug)]
pub struct ServerResponse(String, String); // Only used for intermediate and positive responses (1xx, 2xx, 3xx)

impl From<(&str, &str)> for ServerResponse {
    fn from(tuple: (&str, &str)) -> Self {
        ServerResponse(tuple.0.trim_end().to_string(), tuple.1.to_string())
    }
}

#[derive(Debug, Snafu)]
pub enum Error { // Used for 4xx and 5xx
    #[snafu(display("Server returned negative reply: {} {} from command", response.0, response.1))]
    NegativeReturnCode { response: ServerResponse },
    #[snafu(display("Received malformed data"))]
    InvalidData,
    #[snafu(display("IO error: {}", source))]
    IOError { source: std::io::Error },
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError {source: e}
    }
}

type Result<T, E = self::Error> = std::result::Result<T, E>;

impl From<ServerResponse> for Result<ServerResponse> {
    fn from(response: ServerResponse) -> Self {
        let char = response.0.chars().nth(0).unwrap_or('0');
        match char {
            '1' | '2' | '3' => Ok(response),
            '4' | '5' => Err(Error::NegativeReturnCode { response: response }),
            _ => Err(Error::InvalidData)
        }
    }
}

impl FTPConnection {
    pub fn new(hostname: String, connection_type: ConnectionType) -> std::io::Result<FTPConnection> {
        Ok(FTPConnection { control_stream: TcpStream::connect(hostname + ":21")?, data_stream: None, r#type: connection_type })
    }
    pub fn read_server_response(&mut self) -> self::Result<ServerResponse> {
        let mut res = String::new();
        let mut reader = BufReader::new(&self.control_stream);
        reader.read_line(&mut res)?;
        let response: ServerResponse = res.split_at(4).into();
        println!("{:?}", response);
        response.into()
    }
    pub fn issue_command(&mut self, command: &str, arguments: Vec<&str>) -> self::Result<ServerResponse> {
        self.control_stream.write_fmt(format_args!("{} {}\n", command, arguments.join(" ")))?;
        Ok(self.read_server_response()?)
    }
    pub fn login(&mut self, username: &str, password: &str) -> self::Result<ServerResponse> {
        self.read_server_response()?;
        self.issue_command("USER", vec![username])?;
        Ok(self.issue_command("PASS", vec![password])?)
    }
    pub fn establish_data_connection(&mut self) -> self::Result<ServerResponse> {
        match &self.r#type {
            self::ConnectionType::Passive => {  
                let passive_response = self.issue_command("PASV", vec![])?;
                if passive_response.0.starts_with("227") {
                    let passive_data: Vec<&str> = passive_response.1.split_once('(')
                        .ok_or(Error::InvalidData)?
                        .1
                        .trim_end()
                        .strip_suffix(").")
                        .ok_or(Error::InvalidData)?
                        .split(',').collect();
                    println!("{:?}", passive_data);
                    let address = passive_data[0..4].join(".");
                    let port = (passive_data[4].parse::<u16>().unwrap_or(0) * 0x100 + passive_data[5].parse::<u16>().unwrap_or(0)).to_string();
                    self.data_stream = Some(TcpStream::connect(address + ":" + &port)?);
                    Ok(passive_response)
                }
                else {
                    Err(Error::NegativeReturnCode { response: passive_response })
                }
            }
            self::ConnectionType::Active => { // Do not use!
                let active_response = self.issue_command("PORT", vec![])?;
                Ok(active_response)
            
            }
        }
    }
    pub fn get_directory_listing(&mut self) -> self::Result<Vec<String>> {
        self.establish_data_connection()?;
        self.issue_command("LIST", vec![])?;
        println!("Getting directory!");

        let mut res = String::new();
        self.data_stream.as_ref().unwrap().read_to_string(&mut res)?;
        Ok(res.split('\n').map(|s| s.trim_end().to_string()).collect())
    }
}
