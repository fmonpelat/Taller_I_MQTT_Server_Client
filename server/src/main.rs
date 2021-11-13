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
            if let Err(e) = logger.info("Successfully connect to clients.".to_string()){
                println!("{:?}", e)
            }
        }    
        Err(e) => {
            if let Err(error_log) = logger.info(format!("Unexpected error{:?}", e)){
                println!("{:?}", error_log)
            }
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
