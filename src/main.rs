use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, SelectView, TextView};

use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

/// Set terminal window title using OSC 0 control sequence
/// // OSC 0;title BEL - set terminal window title
fn set_terminal_title(title: &str) {
    print!("\x1b]0;{}\x07", title);
    let _ = io::stdout().flush();
}

#[derive(Debug, Clone)]
struct AgentChoice {
    name: String,
    executable: String,
    description: String,
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

/// Helper to check if a path points to an executable file
fn is_executable(path: &std::path::Path) -> bool {
    if path.is_file() {
        if let Ok(metadata) = path.metadata() {
            return metadata.permissions().mode() & 0o111 != 0;
        }
    }
    false
}

/// Checks if an executable binary exists and has execute permissions in PATH or at absolute path
fn is_in_path(executable: &str) -> bool {
    let exec_path = std::path::Path::new(executable);
    if exec_path.is_absolute() {
        return is_executable(exec_path);
    }

    if let Some(path_var) = env::var_os("PATH") {
        for path in env::split_paths(&path_var) {
            let full_path = path.join(executable);
            if is_executable(&full_path) {
                return true;
            }
        }
    }
    false
}

fn select_agent(available_choices: Vec<AgentChoice>) -> Option<AgentChoice> {
    if available_choices.is_empty() {
        return None;
    }

    let mut siv = cursive::default();
    let selected_choice = Rc::new(RefCell::new(None));

    let mut select_view = SelectView::<AgentChoice>::new().autojump();

    for choice in &available_choices {
        let label = format!(
            "{:<10} - {} ({})",
            choice.name, choice.description, choice.executable
        );
        select_view.add_item(label, choice.clone());
    }

    let selected_choice_clone = Rc::clone(&selected_choice);
    select_view.set_on_submit(move |s, choice| {
        *selected_choice_clone.borrow_mut() = Some(choice.clone());
        s.quit();
    });

    let selected_choice_cancel = Rc::clone(&selected_choice);

    let layout = LinearLayout::vertical()
        .child(TextView::new("Select a coding agent to launch:"))
        .child(DummyView)
        .child(select_view.min_width(50));

    siv.add_layer(
        Dialog::around(layout)
            .title("🚀 Coding Agent Launcher")
            .button("Cancel", move |s| {
                *selected_choice_cancel.borrow_mut() = None;
                s.quit();
            }),
    );

    siv.add_global_callback('q', |s| s.quit());

    siv.run();

    let result = selected_choice.borrow().clone();
    result
}

fn main() {
    set_terminal_title("zed-agent-launcher");

    let shell_path = env::var("SHELL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "/bin/sh".to_string());

    let choices = vec![
        AgentChoice {
            name: "shell".to_string(),
            executable: shell_path.clone(),
            description: format!("System Shell ({})", shell_path),
        },
        AgentChoice {
            name: "pi".to_string(),
            executable: "pi".to_string(),
            description: "pi-coding-agent".to_string(),
        },
        AgentChoice {
            name: "agy".to_string(),
            executable: "agy".to_string(),
            description: "Google Antigravity".to_string(),
        },
        AgentChoice {
            name: "codex".to_string(),
            executable: "codex".to_string(),
            description: "OpenAI Codex".to_string(),
        },
        AgentChoice {
            name: "opencode".to_string(),
            executable: "opencode".to_string(),
            description: "OpenCode".to_string(),
        },
    ];

    let mut available_choices: Vec<AgentChoice> = choices
        .into_iter()
        .filter(|choice| is_in_path(&choice.executable))
        .collect();

    if available_choices.is_empty() {
        eprintln!(
            "✖ Error: None of the supported choices (shell, pi, agy, codex, opencode) were found."
        );
        eprintln!(
            "Please verify that at least one agent or shell is available."
        );
        std::process::exit(1);
    }

    // Sort available choices by MRU (Most Recently Used) order
    let history = load_recent_history();
    available_choices.sort_by_key(|choice| {
        history
            .iter()
            .position(|item| item == &choice.executable || item == &choice.name)
            .unwrap_or(usize::MAX)
    });

    let forwarded_args: Vec<String> = env::args().skip(1).collect();

    let selection = select_agent(available_choices);

    match selection {
        Some(choice) => {
            save_recent_choice(&choice.executable);
            set_terminal_title(&choice.name);

            println!("Launching {} ({}) ...", choice.name, choice.executable);

            let mut cmd = Command::new(&choice.executable);
            cmd.args(&forwarded_args);

            // exec replaces the current process image with the selected agent executable
            let err = cmd.exec();

            // exec only returns if execution failed
            eprintln!(
                "\n✖ Error: Failed to execute '{}': {}",
                choice.executable, err
            );
            std::process::exit(1);
        }
        None => {
            println!("Selection cancelled.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_path_absolute_and_binary() {
        assert!(is_in_path("/bin/sh") || is_in_path("/usr/bin/sh"));
        assert!(is_in_path("sh"));
        assert!(!is_in_path("/nonexistent_path_to_binary_12345"));
    }

    #[test]
    fn test_shell_resolution_fallback() {
        let shell_path = env::var("SHELL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "/bin/sh".to_string());
        assert!(!shell_path.is_empty());
        assert!(is_in_path(&shell_path));
    }
}

