use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval: u64,

    /// Runs once at startup, before the first poll. No PR context, so
    /// template variables are not expanded (commands run verbatim).
    #[serde(default)]
    pub on_start: Vec<ActionCommand>,

    #[serde(default)]
    pub on_new_pr: Vec<ActionCommand>,

    /// Runs every poll cycle against ALL current PRs (not just new ones).
    /// Each hook runs at most once per PR (tracked by repo+number+hook name).
    #[serde(default)]
    pub on_poll: Vec<ActionCommand>,

    /// Runs when a previously tracked PR disappears from the list
    /// (e.g., merged, closed, or review request removed).
    #[serde(default)]
    pub on_remove: Vec<ActionCommand>,

    #[serde(default)]
    pub on_select: Option<SelectCommand>,

    /// Manual actions the user can pick from a popup menu (key `a`) and run
    /// against the selected PR. Each has a display `name` and a `command`
    /// (same `{repo}/{number}/{title}/{author}/{url}/{labels}` templating).
    #[serde(default)]
    pub actions: Vec<ActionCommand>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActionCommand {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SelectCommand {
    pub command: String,
}

fn default_interval() -> u64 {
    120
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval: default_interval(),
            on_start: Vec::new(),
            on_new_pr: Vec::new(),
            on_poll: Vec::new(),
            on_remove: Vec::new(),
            on_select: None,
            actions: Vec::new(),
        }
    }
}

pub fn config_path() -> PathBuf {
    // Check XDG-style ~/.config first (common on Linux and user preference on macOS),
    // then fall back to platform-native config dir (~/Library/Application Support on macOS)
    if let Ok(home) = std::env::var("HOME") {
        let xdg_path = PathBuf::from(&home)
            .join(".config")
            .join("gh-review-watcher")
            .join("config.toml");
        if xdg_path.exists() {
            return xdg_path;
        }
    }

    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("gh-review-watcher")
        .join("config.toml")
}

fn log(msg: &str) {
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/gh-review-watcher.log")
    {
        let now = chrono::Local::now().format("%H:%M:%S");
        let _ = writeln!(f, "[{now}] {msg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_on_start_hooks() {
        let cfg: Config = toml::from_str(
            r#"
            [[on_start]]
            name = "rename"
            command = "herdr workspace rename $HERDR_WORKSPACE_ID gh-review-watcher"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.on_start.len(), 1);
        assert_eq!(cfg.on_start[0].name, "rename");
    }
}

pub fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        match toml::from_str::<Config>(&content) {
            Ok(config) => {
                log(&format!(
                    "Config loaded: interval={}, on_start={} hooks, on_new_pr={} hooks, on_poll={} hooks, on_remove={} hooks, on_select={}, actions={}",
                    config.interval,
                    config.on_start.len(),
                    config.on_new_pr.len(),
                    config.on_poll.len(),
                    config.on_remove.len(),
                    config.on_select.is_some(),
                    config.actions.len()
                ));
                config
            }
            Err(e) => {
                log(&format!("Config parse error: {e}"));
                eprintln!("Warning: failed to parse config: {e}");
                Config::default()
            }
        }
    } else {
        log(&format!("Config not found at {}", path.display()));
        Config::default()
    }
}
