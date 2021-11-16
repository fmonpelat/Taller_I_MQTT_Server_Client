mod logger;
mod server;
use crate::logger::{Logger, Logging};
use crate::server::Server;

fn main() -> Result<(), ()> {
    let file_name = "../log.txt";
    let logger = Logger::new(file_name);

    let server = Server::new("0.0.0.0".to_owned(), "3333".to_owned(), file_name);
    match server.connect() {
        Ok(_) => {
            logger.info("Successfully connect to clients.".to_string());
        }
        Err(e) => {
            logger.info(format!("Unexpected error{:?}", e));
            return Err(());
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