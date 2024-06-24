use rustyline::error::ReadlineError;
use rustyline::{config::Config, Editor};
use shell::Shell;
use std::env;
use std::path::PathBuf;

mod config;
mod script;
mod shell;

fn main() -> Result<(), ReadlineError> {
    let args: Vec<String> = env::args().collect();
    let shell_path = PathBuf::from(args[0].as_str()).canonicalize().unwrap();
    env::set_var("SHELL", shell_path.to_str().unwrap());

    let builder = Config::builder()
        .max_history_size(config::MAX_HISTORY_ENTRIES)
        .unwrap()
        .history_ignore_space(true)
        .history_ignore_dups(true)
        .unwrap()
        .tab_stop(4)
        .indent_size(4);

    let rl = Editor::with_config(builder.build())?;
    let mut shell = Shell::new(config::SHELL_PROMPT.to_string(), rl);

    if args.len() > 1 {
        let script_path = PathBuf::from(&args[1]).canonicalize().unwrap();

        script::execute(script_path, &mut shell);
    }

    ctrlc::set_handler(|| {}).unwrap();

    loop {
        match shell.prompt() {
            Err(ReadlineError::Eof) => break,
            Err(ReadlineError::Interrupted) => shell.ctrlc(),
            Err(_) | Ok(()) => (),
        }
    }

    Ok(())
}
