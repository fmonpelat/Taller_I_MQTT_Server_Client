mod logger;
mod server;
use crate::logger::{Logger, Logging};
use crate::server::Server;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Result};

pub fn get_contents(file_name: &str) -> Result<String> {
    let file = File::open(file_name).expect(&format!("file not found: {}", file_name));
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    return Ok(contents);
}

pub fn load_config(file_name: &str) -> HashMap<String, String> {
    let mut hash_config: HashMap<String, String> = HashMap::new();
    if let Ok(contents) = get_contents(file_name) {
        let lines: Vec<String> = contents.split("\n").map(|s: &str| s.to_string()).collect();
        let mut line_vec: Vec<&str> = vec![];
        for line in &lines {
            line_vec = line.split(": ").collect();
            hash_config.insert(line_vec[0].to_string(), line_vec[1].to_string());
        }
    }
    return hash_config;
}

fn main() -> Result<()> {
    let file_config = "src/config.yaml";
    let config = load_config(file_config);
    let host = config
        .get("host")
        .unwrap_or_else(|| panic!("Cannot found host in config"));
    let port = config
        .get("port")
        .unwrap_or_else(|| panic!("Cannot found port in config"));
    let logfile = config
        .get("logfile")
        .unwrap_or_else(|| panic!("Cannot found logfile in config"));
    let logger = Logger::new(logfile, true);

    let server = Server::new(host.to_owned(), port.to_owned(), logfile);

    match server.listening() {
        Ok(_) => {
            logger.info("Successfully listening to incoming clients.".to_string());
        }
        Err(e) => {
            logger.info(format!("Unexpected error{:?}", e));
            return Err(Error::new(ErrorKind::Other, "Error server"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{get_contents, load_config};

    #[test]

    fn test_sample_server() {
        assert_eq!(1, 1)
    }

    #[test]
    fn test_get_contents() {
        let file_name = "src/config.yaml";
        let contents = get_contents(file_name);
        assert_eq!(
            contents.unwrap(),
            "host: localhost\nport: 3333\nlogfile: ../log.txt"
        )
    }

    #[test]
    fn test_load_config() {
        let mut hash_config_example = HashMap::new();
        hash_config_example.insert("logfile".to_string(), "../log.txt".to_string());
        hash_config_example.insert("port".to_string(), "3333".to_string());
        hash_config_example.insert("host".to_string(), "localhost".to_string());

        let file_name = "src/config.yaml";
        let hash_config = load_config(file_name);
        assert_eq!(hash_config, hash_config_example)
    }
}
