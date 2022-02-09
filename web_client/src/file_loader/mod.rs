use std::io::{BufReader, Read, Result};
use std::{collections::HashMap, fs::File };

pub fn get_contents(file_name: &str) -> Result<String> {
    let file = File::open(file_name).unwrap_or_else(|e| panic!("Error: {}", e));
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn load_contents(file_name: &str) -> HashMap<String, String> {
    let mut hash_config: HashMap<String, String> = HashMap::new();
    if let Ok(contents) = get_contents(file_name) {
        let lines: Vec<String> = contents
            .split('\n')
            .map(|s: &str| s.replace(" ", "").to_string()) // filter whitespaces
            .filter(|lines| !lines.starts_with('#')) // filter comments
            .filter(|s| !s.is_empty()) // filter empty lines
            .collect();
        let mut line_vec: Vec<&str>;
        for line in &lines {
            line_vec = line.split(":").collect();
            hash_config.insert(line_vec[0].to_string(), line_vec[1].to_string());
        }
    }
    hash_config
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::file_loader::{get_contents, load_contents};

    #[test]
    fn test_get_contents() {
        let file_name = "src/config.yaml";
        let contents = get_contents(file_name);
        assert_eq!(
            contents.unwrap(),
            "host: localhost\nport: 3333\nlogfile: ../log.txt\ncredentials_file: ../credentials.yaml"
        )
    }

    #[test]
    fn test_load_config() {
        let mut hash_config_example = HashMap::new();
        hash_config_example.insert("logfile".to_string(), "../log.txt".to_string());
        hash_config_example.insert("port".to_string(), "3333".to_string());
        hash_config_example.insert("host".to_string(), "localhost".to_string());
        hash_config_example.insert(
            "credentials_file".to_string(),
            "../credentials.yaml".to_string(),
        );

        let file_name = "src/config.yaml";
        let hash_config = load_contents(file_name);
        assert_eq!(hash_config, hash_config_example)
    }
}
