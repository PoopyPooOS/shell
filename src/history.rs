use std::{env, fs, io::{self, Write}, path::PathBuf};

pub struct History {
    history_path: PathBuf,
    pub buffer: Vec<String>,
    max_lines: usize,
}

impl History {
    pub fn new(max_lines: usize) -> Self {
        let mut history_path = PathBuf::new();
        history_path.push(env::var("HOME").expect("Failed to get home directory"));
        history_path.push(".shell_history");

        if !history_path.exists() {
            fs::write(&history_path, [0]).expect("Failed to create history file");
        }

        Self { history_path, buffer: Vec::new(), max_lines }
    }

    pub fn read_history(&self) -> Result<Vec<String>, io::Error> {
        let history_text = fs::read_to_string(&self.history_path)?;
        let history: Vec<String> = history_text.lines().map(|s| s.to_string()).collect();

        Ok(history)
    }
    
    pub fn add_history(&mut self, entry: &String) {
        self.buffer.push(entry.to_owned());
        if self.buffer.len() > self.max_lines / 2 {
            self.flush_history().unwrap();
            self.buffer.remove(0);
        }
    }

    pub fn _get_history_entry(_index: usize) -> String {
        String::new()
    }
    
    pub fn flush_history(&mut self) -> Result<(), io::Error> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.history_path)?;

        for entry in &self.buffer {
            file.write(entry.as_bytes())?;
        }
        
        self.buffer.clear();
        Ok(())
    }
}
