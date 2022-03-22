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

#[derive(Debug)]
pub struct ServerResponse(String, String);

impl From<(&str, &str)> for ServerResponse {
    fn from(tuple: (&str, &str)) -> Self {
        ServerResponse(tuple.0.trim_end().to_string(), tuple.1.to_string())
    }
}

impl FTPConnection {
    pub fn new(hostname: String, connection_type: ConnectionType) -> std::io::Result<FTPConnection> {
        Ok(FTPConnection { control_stream: TcpStream::connect(hostname + ":21")?, data_stream: None, r#type: connection_type })
    }
    pub fn read_server_response(&mut self) -> std::io::Result<ServerResponse> {
        let mut res = String::new();
        let mut reader = BufReader::new(&self.control_stream);
        reader.read_line(&mut res)?;
        let tmp = res.split_at(4);

        Ok(tmp.into())
    }
    pub fn issue_command(&mut self, command: &str, arguments: Vec<&str>) -> std::io::Result<ServerResponse> {
        self.control_stream.write_fmt(format_args!("{} {}\n", command, arguments.join(" ")))?;
        Ok(self.read_server_response()?)
    }
    pub fn login(&mut self, username: &str, password: &str) -> std::io::Result<ServerResponse> {
        if self.read_server_response()?.0.starts_with("220") == false {
            return Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Server is not ready for new connection!"))
        }
        self.issue_command("USER", vec![username])?;
        Ok(self.issue_command("PASS", vec![password])?)
    }
    pub fn establish_data_connection(&mut self) -> std::io::Result<()> {
        match &self.r#type {
            self::ConnectionType::Passive => {  
                let passive_response = self.issue_command("PASV", vec![])?;
                if passive_response.0.starts_with("227") {
                    let passive_data: Vec<&str> = passive_response.1.split_once('(')
                        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not get parameters for Passive Mode! Try using Active Mode?"))?
                        .1
                        .trim_end()
                        .strip_suffix(").")
                        .ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not get parameters for Passive Mode! Try using Active Mode?"))?
                        .split(',').collect();
                    println!("{:?}", passive_data);
                    let address = passive_data[0..4].join(".");
                    let port = (passive_data[4].parse::<u16>().unwrap_or(0) * 0x100 + passive_data[5].parse::<u16>().unwrap_or(0)).to_string();
                    self.data_stream = Some(TcpStream::connect(address + ":" + &port)?);
                    Ok(())
                }
                else {
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not establish Passive Mode connection!"))
                }
            }
            self::ConnectionType::Active => {

                Ok(())
            }
        }
    }
    pub fn get_directory_listing(&mut self) -> std::io::Result<Vec<String>> {
        self.establish_data_connection()?;
        self.issue_command("LIST", vec![])?;
        let mut res = String::new();
        self.data_stream.as_ref().unwrap().read_to_string(&mut res)?;
        Ok(res.split('\n').map(|s| s.trim_end().to_string()).collect())
    }
}
