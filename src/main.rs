use std::env;
use std::fmt;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use colored::Colorize;
use inquire::Select;

#[derive(Debug, Clone)]
struct AgentChoice {
    name: &'static str,
    executable: &'static str,
    description: &'static str,
}

impl fmt::Display for AgentChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<10} - {} ({})",
            self.name.cyan().bold(),
            self.description,
            self.executable.dimmed()
        )
    }
}

fn get_history_file_path() -> Option<PathBuf> {
    let base = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")));
    base.map(|b| b.join("zed-agent-launcher").join("recent.txt"))
}

fn load_recent_history() -> Vec<String> {
    if let Some(path) = get_history_file_path() {
        if let Ok(content) = fs::read_to_string(path) {
            return content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

fn save_recent_choice(executable: &str) {
    if let Some(path) = get_history_file_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut history = load_recent_history();
        history.retain(|item| item != executable);
        history.insert(0, executable.to_string());
        let _ = fs::write(path, history.join("\n"));
    }
}

/// Checks if an executable binary exists and has execute permissions in PATH
fn is_in_path(executable: &str) -> bool {
    if let Some(path_var) = env::var_os("PATH") {
        for path in env::split_paths(&path_var) {
            let full_path = path.join(executable);
            if full_path.is_file() {
                if let Ok(metadata) = full_path.metadata() {
                    if metadata.permissions().mode() & 0o111 != 0 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn main() {
    let choices = vec![
        AgentChoice {
            name: "pi",
            executable: "pi",
            description: "pi-coding-agent",
        },
        AgentChoice {
            name: "agy",
            executable: "agy",
            description: "Google Antigravity",
        },
        AgentChoice {
            name: "codex",
            executable: "codex",
            description: "OpenAI Codex",
        },
        AgentChoice {
            name: "opencode",
            executable: "opencode",
            description: "OpenCode",
        },
    ];

    let mut available_choices: Vec<AgentChoice> = choices
        .into_iter()
        .filter(|choice| is_in_path(choice.executable))
        .collect();

    if available_choices.is_empty() {
        eprintln!(
            "{} None of the supported coding agents (pi, agy, codex, opencode) were found in your PATH.",
            "✖ Error:".red().bold()
        );
        eprintln!("Please verify that at least one agent is installed and available in your system PATH.");
        std::process::exit(1);
    }

    // Sort available choices by MRU (Most Recently Used) order
    let history = load_recent_history();
    available_choices.sort_by_key(|choice| {
        history
            .iter()
            .position(|item| item == choice.executable)
            .unwrap_or(usize::MAX)
    });

    let forwarded_args: Vec<String> = env::args().skip(1).collect();

    println!("{}", "🚀 Coding Agent Launcher".bold().green());

    let selection = Select::new("Select a coding agent to launch:", available_choices)
        .with_help_message("Type keywords to filter options, ↑↓ to move, Enter to select")
        .prompt();

    match selection {
        Ok(choice) => {
            save_recent_choice(choice.executable);

            println!(
                "Launching {} ({}) ...",
                choice.name.green().bold(),
                choice.executable.cyan()
            );

            let mut cmd = Command::new(choice.executable);
            cmd.args(&forwarded_args);

            // exec replaces the current process image with the selected agent executable
            let err = cmd.exec();

            // exec only returns if execution failed
            eprintln!(
                "\n{} Failed to execute '{}': {}",
                "✖ Error:".red().bold(),
                choice.executable,
                err
            );
            std::process::exit(1);
        }
        Err(_) => {
            println!("Selection cancelled.");
        }
    }
}
