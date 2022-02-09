
use std::fmt;
use std::error::Error;


pub struct Request<'a> {
    method: &'a str,
    uri: &'a str,
    http_version: &'a str,
}

impl<'a> fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\r\n",
            self.method,
            self.uri,
            self.http_version
        )
    }
}


#[allow(dead_code)]
pub fn parse_request_line(request: &str) -> Result<Request, Box<dyn Error>> {
    let mut parts = request.split_whitespace();
    
    let method = parts.next().ok_or("Method not specified")?;
    // We only accept GET requests
    if method != "GET" {
        Err("Unsupported method")?;
    }

    let uri = parts.next().ok_or("URI not specified").expect("Invalid unicode!");

    let http_version = parts.next().ok_or("HTTP version not specified")?;
    if http_version != "HTTP/1.1" {
        Err("Unsupported HTTP version, use HTTP/1.1")?;
    }

    if uri != "/" {
        Err("Unsupported URI")?;
    }

    Ok(Request {
        method,
        uri,
        http_version,
    })
}