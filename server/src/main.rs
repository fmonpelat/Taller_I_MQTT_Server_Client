mod file_loader;
mod logger;
mod server;
use crate::file_loader::load_contents;
use crate::logger::{Logger, Logging};
use crate::server::Server;
use std::io::{Error, ErrorKind, Result};

fn main() -> Result<()> {
    let file_config = "src/config.yaml";
    let config = load_contents(file_config);
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

    #[test]
    fn test_sample_server() {
        assert_eq!(1, 1)
    }
}
