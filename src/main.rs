use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, SelectView, TextView};

use std::cell::RefCell;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct AgentChoice {
    name: &'static str,
    executable: &'static str,
    description: &'static str,
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
            "✖ Error: None of the supported coding agents (pi, agy, codex, opencode) were found in your PATH."
        );
        eprintln!(
            "Please verify that at least one agent is installed and available in your system PATH."
        );
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

    let selection = select_agent(available_choices);

    match selection {
        Some(choice) => {
            save_recent_choice(choice.executable);

            println!("Launching {} ({}) ...", choice.name, choice.executable);

            let mut cmd = Command::new(choice.executable);
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
