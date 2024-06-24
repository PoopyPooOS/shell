use crate::config;
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use rustyline::{error::ReadlineError, history::DefaultHistory, Editor};
use std::{
    env,
    io::{self, ErrorKind},
    path::PathBuf,
    process::{self, Command, Stdio},
};

pub struct Shell {
    active_directory: PathBuf,
    current_task: Option<Pid>,
    rl: Editor<(), DefaultHistory>,
    prompt: String,
}

impl Shell {
    #[must_use]
    pub fn new(prompt: String, rl: Editor<(), DefaultHistory>) -> Self {
        let active_directory = env::current_dir().expect("Failed to get current directory");

        Self {
            active_directory,
            current_task: None,
            prompt,
            rl,
        }
    }

    pub fn prompt(&mut self) -> Result<(), ReadlineError> {
        let pwd = self.active_directory.canonicalize().unwrap();

        let input = self.rl.readline(&format!("{} {} ", pwd.display(), self.prompt))?;

        self.rl.add_history_entry(&input)?;
        self.execute_command(&input)?;

        Ok(())
    }

    pub fn ctrlc(&self) {
        if self.current_task.is_none() {
            return;
        }

        kill(self.current_task.unwrap(), Signal::SIGINT).expect("Failed to send SIGINT to running task");
    }

    fn change_directory(&mut self, mut path: PathBuf) {
        if !path.is_absolute() {
            let current_dir = self.active_directory.canonicalize().expect("Failed to canonicalize cwd");

            path = current_dir.join(path);
        }

        match path {
            nonexistent if !nonexistent.exists() => eprintln!("Directory does not exist."),
            file if file.is_file() => eprintln!("Not a directory."),
            _ => self.active_directory = path,
        }
    }

    pub fn execute_command(&mut self, command: &str) -> io::Result<()> {
        if command.is_empty() {
            return Ok(());
        }

        let parts = command.split_whitespace().collect::<Vec<&str>>();
        let command_name = parts[0];
        let mut args: Vec<String> = Vec::new();
        let mut env_vars: Vec<(String, String)> = Vec::new();

        for part in parts {
            if let Some(index) = part.find('=') {
                let (key, value) = part.split_at(index);
                let key = key.trim_start_matches('$').to_string();
                let value = value.trim_start_matches('=').to_string();

                env_vars.push((key, value));
            } else {
                let processed = self.process_env_vars(part);

                if part != command_name {
                    args.push(processed);
                }
            }
        }

        for (key, value) in &env_vars {
            env::set_var(key, value);
        }

        if !env_vars.is_empty() {
            return Ok(());
        }

        args = args.into_iter().map(|arg| self.process_env_vars(&arg)).collect();

        let is_background = args.last().is_some_and(|x| x == "&");
        if is_background {
            args.pop();
        }

        match command_name {
            "cd" => {
                if args.is_empty() {
                    self.change_directory(PathBuf::from(env::var("HOME").expect("Failed to get home directory")));
                } else {
                    self.change_directory(PathBuf::from(args[0].clone()));
                }
            }
            "exit" => process::exit(0),
            "unset" => {
                if args.is_empty() {
                    eprintln!("Usage: unset <variable>");
                    return Ok(());
                }

                env::remove_var(&args[0]);
            }
            "help" => println!("{}", config::HELP.trim_start_matches('\n').trim_end_matches('\n')),
            _ => {
                let child = Command::new(command_name)
                    .args(&args)
                    .current_dir(&self.active_directory)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn();

                if child.is_err() {
                    let err = child.unwrap_err();
                    match err.kind() {
                        ErrorKind::NotFound => {
                            eprintln!("Command not found: {command_name}");
                        }
                        _ => eprintln!("{err:#?}"),
                    }
                    return Ok(());
                }

                let mut child = child.unwrap();

                if is_background {
                    println!("Background task started with PID: {}", child.id());
                } else {
                    self.current_task = Some(Pid::from_raw(child.id().try_into().unwrap()));
                    child.wait()?;
                    self.current_task = None;
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn process_env_vars(&self, arg: &str) -> String {
        if arg.starts_with('$') {
            if let Ok(var) = env::var(arg.trim_start_matches('$')) {
                return var;
            }
        }

        arg.to_string()
    }
}
