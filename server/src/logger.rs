use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub trait Logging {
    fn log(&self, msg: String);
    fn new(file_name:&str,source_log:&str)->Self;
}


pub type Logger = Arc<Mutex<BufWriter<File>>>;
impl Logging for Logger{
    fn new(file_name:&str, source_log:&str) -> Self {
        let file = match OpenOptions::new()
                .read(false)
                .append(true)
                .write(true)
                .create(true)
                .open(source_log.to_owned()+file_name)
            {
                Err(_file) => panic!("Unable to open log file "),
                Ok(file) => file,
            };
            Arc::new(Mutex::new(BufWriter::new(file)))   
    }

    fn log(&self, message: String) {
        match self
            .lock()
            .unwrap()
            .write(format!("{}\n", message).as_bytes())
        {
            Err(_log_file) => panic!("Unable to writing log file "),
            Ok(_log_file) => (),
        }
    }
}