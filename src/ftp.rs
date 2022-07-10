extern crate snafu;

use snafu::prelude::*;
use std::io::prelude::*;
use std::net::TcpStream;
use std::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncBufReadExt};

pub enum ConnectionType {
    Passive,
    Active
}

pub struct Connection {
    control_stream: TcpStream,
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
    #[snafu(display("Server returned negative reply: {} {}", response.0, response.1))]
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

pub type Result<T, E = self::Error> = std::result::Result<T, E>;

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

pub enum TransferMode {
    ASCII, 
    Binary, 
    EBCDIC,
    Unicode // Not officially supported, experimental
}

impl Connection {
    pub fn new(hostname: &str, connection_type: ConnectionType) -> self::Result<Connection> {
        Ok(Connection { control_stream: TcpStream::connect(hostname)?, r#type: connection_type })
    }
    pub fn read_server_response(&mut self) -> self::Result<ServerResponse> {
        let mut res = String::new();
        let mut reader = BufReader::new(&self.control_stream);
        reader.read_line(&mut res)?;
        let response: ServerResponse = res.split_at(4).into();

        response.into()
    }
    pub async fn async_read_server_response(&mut self) -> self::Result<ServerResponse> {
        let mut res = String::new();
        let mut reader = tokio::io::BufReader::new(tokio::net::TcpStream::from_std(self.control_stream.try_clone()?)?);
        reader.read_line(&mut res).await?;
        let response: ServerResponse = res.split_at(4).into();

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
    pub fn establish_data_connection(&mut self) -> self::Result<TcpStream> {
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
                    let address = passive_data[0..4].join(".");
                    let port = (passive_data[4].parse::<u16>().unwrap_or(0) * 0x100 + passive_data[5].parse::<u16>().unwrap_or(0)).to_string();
                    Ok(TcpStream::connect(address + ":" + &port)?)
                }
                else {
                    Err(Error::NegativeReturnCode { response: passive_response })
                }
            }
            self::ConnectionType::Active => { // Do not use!
                /*let active_response = self.issue_command("PORT", vec![])?;
                Ok(active_response)*/
                Err(Error::InvalidData)
            }
        }
    }
    pub fn get_directory_listing(&mut self) -> self::Result<Vec<String>> {
        let mut stream = self.establish_data_connection()?;
        self.issue_command("NLST", vec![])?;

        let mut res = String::new();
        stream.read_to_string(&mut res)?;
        self.read_server_response()?;

        Ok(res.split('\n').map(|s| s.trim_end().to_string()).filter(|s| s.len() > 0).collect())
    }
    pub fn set_transfer_mode(&mut self, mode: TransferMode) -> self::Result<ServerResponse> {
        self.issue_command("TYPE", vec![
            match mode {
                TransferMode::ASCII => "A",
                TransferMode::Binary => "I",
                TransferMode::EBCDIC => "E",
                TransferMode::Unicode => "U" // Not implemented on all servers
            }
        ])
    }

    pub async fn receive_file(&mut self, filename: &str) -> self::Result<Vec<u8>> {
        let mut stream = tokio::net::TcpStream::from_std(self.establish_data_connection()?)?;
        let mut res = Vec::new();
        self.issue_command("RETR", vec![filename])?;

        stream.read_to_end(&mut res).await?;
        self.async_read_server_response().await?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::ftp;
    use std::fs::{self, File};
    use tokio::runtime::Runtime;
    use std::io::prelude::{Write};

    static FTP_URL: &str = "ftp.dlptest.com:21";
    static FTP_USER: &str = "dlpuser";
    static FTP_PASS: &str = "rNrKYTX9g7z3RgJRmxWuGHbeu";
    
    fn test_login() -> ftp::Result<ftp::Connection> {
        let mut ftp = ftp::Connection::new(FTP_URL, ftp::ConnectionType::Passive)?;
        ftp.login(FTP_USER, FTP_PASS)?;
        Ok(ftp)
    }

    #[test]
    fn login_test() -> ftp::Result<()> {
        // Log onto DLP test server
        test_login()?;
        Ok(())
    }
    #[test]
    fn data_connection_test() -> ftp::Result<()> {
        // Log onto DLP test server
        let mut ftp = test_login()?;

        // Establish the data connection
        ftp.establish_data_connection()?;

        Ok(())
    }
    #[test]
    fn list_remote_files() -> ftp::Result<()> {
        // Log onto DLP test server
        let mut ftp = test_login()?;
        let files = ftp.get_directory_listing()?;

        // Listing is not empty
        assert_ne!(files.len(), 0);

        // File names are not empty
        for f in files {
            assert_ne!(f.len(), 0);
        }

        Ok(())
    }
    #[test]
    fn file_download_test() -> ftp::Result<()> {
        // Log onto DLP test server
        let mut ftp = ftp::Connection::new(FTP_URL, ftp::ConnectionType::Passive)?;
        ftp.login(FTP_USER, FTP_PASS)?;

        // Create temporary file
        let files = ftp.get_directory_listing()?;
        let mut path = files[0].clone();
        path.insert_str(0, "/tmp/");

        let bytes_written;
        // Create a scope for the file
        {
            let mut file = File::create(&path)?;

            // Write file
            let mut rt = Runtime::new()?;
            bytes_written = rt.block_on(async {
                let data = ftp.receive_file(&files[0]).await?;
                file.write(&data).map_err(|e| ftp::Error::from(e))
            })?;
        }   

        let file = File::open(&path)?;
        let metadata = file.metadata()?;

        assert!(metadata.is_file());
        assert_eq!(metadata.len(), bytes_written as u64);

        Ok(())
    }
}