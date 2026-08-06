use std::env;
use std::fmt;
use std::os::unix::process::CommandExt;
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

    let forwarded_args: Vec<String> = env::args().skip(1).collect();

    println!("{}", "🚀 Coding Agent Launcher".bold().green());

    let selection = Select::new("Select a coding agent to launch:", choices)
        .with_help_message("Type keywords to filter options, ↑↓ to move, Enter to select")
        .prompt();

    match selection {
        Ok(choice) => {
            println!(
                "Launching {} ({}) ...",
                choice.name.green().bold(),
                choice.executable.cyan()
            );

            let mut cmd = Command::new(choice.executable);
            cmd.args(&forwarded_args);

            // exec replaces the current process image with the selected agent executable
            let err = cmd.exec();

            // exec only returns if execution failed (e.g. binary not found in PATH)
            eprintln!(
                "\n{} Failed to execute '{}': {}",
                "✖ Error:".red().bold(),
                choice.executable,
                err
            );
            eprintln!(
                "Please verify that '{}' is installed and available in your system PATH.",
                choice.executable.yellow()
            );
            std::process::exit(1);
        }
        Err(_) => {
            println!("Selection cancelled.");
        }
    }
}
