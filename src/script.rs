use std::{fs, path::PathBuf, process};

use crate::shell::Shell;

pub fn execute(path: PathBuf, shell: &mut Shell) {
    let (script_exists, script_is_dir) = (path.exists(), path.is_dir());

    if script_exists && !script_is_dir {
        let script = fs::read_to_string(path).expect("Failed to read script.");
        for (index, line) in script.lines().enumerate() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            match shell.execute_command(line) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error executing script on line {}: {}", index + 1, err);
                    process::exit(1);
                }
            }
        }
    } else {
        if !script_exists {
            eprintln!("File does not exist.");
        } else if script_is_dir {
            eprintln!("Cannot execute a directory.");
        }

        process::exit(1);
    }

    process::exit(0);
}
