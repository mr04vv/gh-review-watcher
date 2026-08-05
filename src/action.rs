use crate::github::PullRequest;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Command, Stdio};

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn expand_template(template: &str, pr: &PullRequest) -> String {
    template
        .replace("{repo}", pr.repo())
        .replace("{number}", &pr.number.to_string())
        .replace("{title}", &shell_escape(&pr.title))
        .replace("{author}", pr.author())
        .replace("{url}", &pr.url)
        .replace("{labels}", &shell_escape(&pr.labels_str()))
}

fn log(msg: &str) {
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/gh-review-watcher.log")
    {
        let now = chrono::Local::now().format("%H:%M:%S");
        let _ = writeln!(f, "[{now}] {msg}");
    }
}

pub fn run_command(command: &str, pr: &PullRequest) {
    spawn_shell(expand_template(command, pr));
}

/// Runs a command verbatim, with no PR context and no template expansion.
/// Used by hooks that fire outside of any PR (e.g. `on_start`).
pub fn run_shell(command: &str) {
    spawn_shell(command.to_string());
}

fn spawn_shell(expanded: String) {
    log(&format!("Running: {expanded}"));
    match Command::new("sh")
        .arg("-c")
        .arg(&expanded)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            // Wait for the child in a separate thread to capture stderr
            std::thread::spawn(move || {
                match child.wait_with_output() {
                    Ok(output) => {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            log(&format!("Command failed (exit {}): {}", output.status, stderr));
                        } else {
                            log("Command succeeded");
                        }
                    }
                    Err(e) => {
                        log(&format!("Failed to wait for command: {e}"));
                    }
                }
            });
        }
        Err(e) => {
            log(&format!("Failed to spawn command: {e}\nCommand: {expanded}"));
        }
    }
}
