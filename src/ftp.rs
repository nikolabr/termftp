extern crate snafu;
extern crate lazy_static;

use snafu::prelude::*;
use std::io::prelude::*;
use std::net::TcpStream;
use std::io::BufReader;

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
    #[snafu(display("Data race"))]
    RaceError
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

impl Drop for Connection {
    fn drop(&mut self) {
        // self.drain_connection();
        self.close().unwrap();
    }
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

    pub fn issue_command(&mut self, command: &str, arguments: Vec<&str>) -> self::Result<ServerResponse> {
        self.control_stream.write_fmt(format_args!("{} {}\n", command, arguments.join(" ")))?;
        Ok(self.read_server_response()?)
    }

    pub fn login(&mut self, username: &str, password: &str) -> self::Result<ServerResponse> {
        self.read_server_response()?;
        self.issue_command("USER", vec![username])?;
        Ok(self.issue_command("PASS", vec![password])?)
    }
    pub fn close(&mut self) -> self::Result<()> {
        self.issue_command("QUIT", vec![])?;
        Ok(())   
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
                    TcpStream::connect(address + ":" + &port).map_err(|e| Error::IOError { source: e })
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

    pub fn receive_file(&mut self, filename: &str) -> self::Result<Vec<u8>> {
        let mut stream = self.establish_data_connection()?;
        let mut res = Vec::new();
        self.issue_command("RETR", vec![filename])?;

        stream.read_to_end(&mut res)?;
        self.read_server_response()?;

        Ok(res)
    }

    pub fn upload_file(&mut self, data: &[u8], filename: &str) -> self::Result<ServerResponse> {
        {
            let mut stream = self.establish_data_connection()?;
            self.issue_command("STOR", vec![filename])?;
            stream.write_all(data)?;
        }
        self.read_server_response()
    }

    pub fn get_remote_size(&mut self, filename: &str) -> self::Result<u64> {
        self.issue_command("SIZE", vec![filename])?.1.trim().parse::<u64>().map_err(|_| Error::InvalidData)
    }

    pub fn delete_file(&mut self, name: &str) -> self::Result<ServerResponse> {
        self.issue_command("DELE", vec![name])
    }

    pub fn make_directory(&mut self, name: &str) -> self::Result<ServerResponse> {
        self.issue_command("MKD", vec![name])
    }

    pub fn change_directory(&mut self, name: &str) -> self::Result<ServerResponse> {
        self.issue_command("CWD", vec![name])
    }

    pub fn remove_directory(&mut self, name: &str) -> self::Result<ServerResponse> {
        self.issue_command("RMD", vec![name])
    }

    pub fn root_directory(&mut self) -> self::Result<ServerResponse> {
        self.issue_command("CDUP", vec![])
    }
}

#[cfg(test)]
mod tests {
    use crate::ftp;
    use std::fs::{File};
    use std::io::prelude::{Write};
    use lazy_static::lazy_static;
    use std::sync::Mutex;

    static FTP_URL: &str = "ftp.dlptest.com:21";
    static FTP_USER: &str = "dlpuser";
    static FTP_PASS: &str = "rNrKYTX9g7z3RgJRmxWuGHbeu";
    
    // Needed to prevent the tests from running in parallel, thus causing a data race
    lazy_static! {
        static ref FTP_MUTEX: Mutex<()> = Mutex::new(());
    }

    #[inline(always)]
    fn test_login() -> ftp::Result<ftp::Connection> {
        let _guard = FTP_MUTEX.lock().map_err(|_| ftp::Error::RaceError)?;
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
        let mut ftp = test_login()?;

        // Create temporary file
        let files = ftp.get_directory_listing()?;
        let mut path = files[0].clone();
        path.insert_str(0, "/tmp/");

        let bytes_written;
        // Create a scope for the file
        {
            let mut file = File::create(&path)?;

            // Write file
            let data = ftp.receive_file(&files[0])?;

            bytes_written = file.write(&data).map_err(|e| ftp::Error::from(e))?;
        }   

        let file = File::open(&path)?;
        let metadata = file.metadata()?;

        assert!(metadata.is_file());
        assert_eq!(metadata.len(), bytes_written as u64);

        Ok(())
    }
    #[test]
    fn file_upload_test() -> ftp::Result<()> {
        let string = "This is a test file";

        // Log onto DLP test server
        let mut ftp = test_login()?;

        // Create test string
        let b = string.as_bytes();
        
        println!("Uploading file");
        // Upload file
        println!("{:?}", ftp.upload_file(b, "test_file")?);
        println!("Uploaded file");

        // Retrieve file from server
        let files = ftp.get_directory_listing()?;

        let pos = files.iter().position(|s| s.as_str() == "test_file").ok_or(ftp::Error::InvalidData)?;
        println!("Receiving file");
        let remote = String::from_utf8(ftp.receive_file(&files[pos])?).map_err(|_| ftp::Error::InvalidData)?;

        assert_eq!(remote, string);

        Ok(())
    }

    #[test]
    fn file_deletion_test() -> ftp::Result<()> {
        // Log onto DLP test server
        let mut ftp = test_login()?;

        // Create test string
        let string = "This is a test file";
        let b = string.as_bytes();
        
        // Upload file
        println!("{:?}", ftp.upload_file(b, "test_file")?);
        
        println!("{:?}", ftp.delete_file("test_file")?);
        Ok(())
    }

    #[test]
    fn remote_size_test() -> ftp::Result<()> {
        // Log onto DLP test server
        let mut ftp = test_login()?;

        let string = "This is a test file";
        let b = string.as_bytes();
        
        // Upload file
        println!("{:?}", ftp.upload_file(b, "test_file")?);
        
        let size = ftp.get_remote_size("test_file")? as usize;

        assert_eq!(size, b.len());
        println!("{}", size);

        Ok(())
    }

    #[test]
    fn change_directory_test() -> ftp::Result<()> {
        let mut ftp = test_login()?;

        let files = ftp.get_directory_listing()?;
        println!("{:?}", files);

        // Remove old test directory if present
        if files.iter().any(|s| s.as_str() == "this_is_a_test_directory") {
            ftp.remove_directory("this_is_a_test_directory")?;
        }

        // Make new directory and CWD into it
        ftp.make_directory("this_is_a_test_directory")?;
        ftp.change_directory("this_is_a_test_directory")?;

        let files2 = ftp.get_directory_listing()?;

        assert_eq!(files2.len(), 0);

        assert_ne!(files, files2);

        Ok(())
    }

    #[test]
    fn root_directory_test() -> ftp::Result<()> {
        let mut ftp = test_login()?;

        let files = ftp.get_directory_listing()?;

        // Remove old test directory if present
        if files.iter().any(|s| s == "this_is_a_test_directory") {
            ftp.remove_directory("this_is_a_test_directory")?;
        }

        ftp.make_directory("this_is_a_test_directory")?;
        ftp.change_directory("this_is_a_test_directory")?;

        let files2 = ftp.get_directory_listing()?;

        ftp.root_directory()?;

        assert_eq!(files2.len(), 0);

        assert_ne!(files, files2);
        
        Ok(())
    }
}