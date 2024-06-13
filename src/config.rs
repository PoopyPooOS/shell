// TODO: Add support for user-defined configs
pub const SHELL_PROMPT: &str = "\n->";
pub const MAX_HISTORY_ENTRIES: usize = 50000;

pub const HELP: &str = r#"
Commands:
    clear, echo, micro, btop, printenv, sleep, create, ls, mount, poweroff, reboot, view

Shell commands:
    cd, exit, help, unset
"#;
