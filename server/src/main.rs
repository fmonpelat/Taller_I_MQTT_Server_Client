mod logger;
mod server;
use crate::logger::{Logger, Logging};
use crate::server::Server;

fn main() {
    let file_name = "../log.txt";
    let logger = Logger::new(file_name);

    let server = Server::new("0.0.0.0","3333", file_name);
    server.connect();

    logger.info("Server terminated.".to_string());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_server() {
        assert_eq!(1, 1)
    }
}
