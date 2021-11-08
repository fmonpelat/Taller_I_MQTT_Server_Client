use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{Result,Error};

pub struct Logger {
    file: Arc<Mutex<BufWriter<File>>>,
    _file_source: String,
}
pub trait Logging {
    fn new(file_source:&str) -> Logger;
    fn log(&self, msg: String) -> Result< Error>;
    fn debug(&self, message: String);
    fn error(&self, message: String);
    fn info(&self, message: String);
}

impl Logging for Logger{
    fn new(file_source:&str) -> Logger {
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
           Logger { file: Arc::new(Mutex::new(BufWriter::new(file))), _file_source: file_source.to_owned() }
    }

    fn log(&self, message: String) -> Result<()> {
        let start = SystemTime::now();
        let timestamp_str = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        match self.file
        .lock()
        {
            Ok(mut file) => {
                file.write_all(format!("{} {}\n", timestamp_str.as_secs(), message).as_bytes());
                file.flush()?;
                Ok(())   

            }
            Err(_) => panic!("Unable to lock file"),
        }
        //return 
    } 
    
    fn debug(&self, message: String) {
        self.log("[DEBUG] ".to_string() + &message);
    }

    fn error(&self, message: String) {
        self.log("[ERROR] ".to_string() + &message);
    }

    fn info(&self, message: String) {
        self.log("[INFO] ".to_string() + &message);
    }
}