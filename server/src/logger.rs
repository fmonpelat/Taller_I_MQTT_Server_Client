use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{Result,Error};

pub struct Logger {
    file: Arc<Mutex<BufWriter<File>>>,
    _file_source: String,
    debug: bool,
}
pub trait Logging {
    fn new(file_source:&str, debug: bool) -> Self;
    fn log(&self, msg: String) -> Result<&'static str> ;
    fn debug(&self, message: String) -> Option<&str> ;
    fn error(&self, message: String)-> Option<&str> ;
    fn info(&self, message: String)-> Option<&str> ;
}

impl Logging for Logger{
    fn new(file_source:&str, debug: bool) -> Logger {
        let file = match OpenOptions::new()
                .read(false)
                .append(true)
                .write(true)
                .create(true)
                .open(file_source)
            {
                Err(_file) => panic!("Unable to open log file "),
                Ok(file) => file,
            };
           Logger { file: Arc::new(Mutex::new(BufWriter::new(file))), _file_source: file_source.to_owned(), debug }
    }

    fn log(&self, message: String) -> Result<&'static str> {
        let start = SystemTime::now();
        let timestamp_str = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        match self.file
        .lock()
        {
            Ok(mut file) => {
                file.write_all(format!("{} {}\n", timestamp_str.as_secs(), message).as_bytes())?;
                file.flush()?;
                if self.debug {
                    println!("{} {}", timestamp_str.as_secs(), message);
                }
                Ok("return to log")   
            }
            Err(_) => Err(Error::new(ErrorKind::Other, "Error logging")),
        }  
    } 
    
    fn debug(&self, message: String) -> Option<&str> {
        self.log("[DEBUG] ".to_string() + &message).ok()
    }

    fn error(&self, message: String) -> Option<&str> {
        self.log("[ERROR] ".to_string() + &message).ok()
    }

    fn info(&self, message: String) -> Option<&str>{
        self.log("[INFO] ".to_string() + &message).ok()
    }
}