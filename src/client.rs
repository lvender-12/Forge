use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

use crate::common::protocol::{Request, Response};

const SOCKET_PATH: &str = "/tmp/forge.sock";

pub fn send(request: Request) -> Result<Response, Box<dyn Error>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    let mut payload = serde_json::to_vec(&request)?;
    payload.push(b'\n');
    stream.write_all(&payload)?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;
    Ok(serde_json::from_str(&response)?)
}
