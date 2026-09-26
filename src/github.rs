use serde::Deserialize;
use std::collections::HashSet;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrKind {
    Review,
    Assignee,
}

impl PrKind {
    pub fn label(&self) -> &'static str {
        match self {
            PrKind::Review => "Review",
            PrKind::Assignee => "Assignee",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
struct RawPullRequest {
    pub repository: RepoInfo,
    pub number: u64,
    pub title: String,
    pub author: AuthorInfo,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub url: String,
    #[serde(default)]
    pub labels: Vec<LabelInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PullRequest {
    pub repository: RepoInfo,
    pub number: u64,
    pub title: String,
    pub author: AuthorInfo,
    pub updated_at: String,
    pub url: String,
    pub labels: Vec<LabelInfo>,
    pub kind: PrKind,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct RepoInfo {
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct AuthorInfo {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct LabelInfo {
    pub name: String,
    #[serde(default)]
    pub color: String,
}

impl PullRequest {
    pub fn repo(&self) -> &str {
        &self.repository.name_with_owner
    }

    pub fn author(&self) -> &str {
        &self.author.login
    }

    /// Format updatedAt for display (truncate to minutes)
    pub fn updated_short(&self) -> &str {
        if self.updated_at.len() >= 16 {
            &self.updated_at[..16]
        } else {
            &self.updated_at
        }
    }

    /// Format labels as comma-separated string
    pub fn labels_str(&self) -> String {
        self.labels
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn search_prs(filter: &str) -> Result<Vec<RawPullRequest>, String> {
    let output = Command::new("gh")
        .args([
            "search",
            "prs",
            filter,
            "--state=open",
            "--json",
            "repository,number,title,author,updatedAt,url,labels",
            "--limit",
            "100",
        ])
        .output()
        .map_err(|e| format!("Failed to run gh: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh command failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse JSON: {e}"))
}

fn ensure_access(
    repos: &[String],
    probe: impl Fn(&str) -> Result<(), String>,
) -> Result<(), String> {
    repos
        .iter()
        .try_for_each(|repo| probe(repo).map_err(|e| format!("cannot access {repo}: {e}")))
}

fn probe_repo(repo: &str) -> Result<(), String> {
    let output = Command::new("gh")
        .args(["api", &format!("repos/{repo}"), "--silent"])
        .output()
        .map_err(|e| format!("Failed to run gh: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn fetch_review_requests(access_check_repos: &[String]) -> Result<Vec<PullRequest>, String> {
    ensure_access(access_check_repos, probe_repo)?;
    let reviews = search_prs("--review-requested=@me")?;
    // An empty review search (e.g. right after wake-from-sleep) is treated as a
    // transient failure. Otherwise assignee-only results bypass the watcher's
    // empty-response guard, every review PR fires on_remove, and all of them
    // are re-detected as new on the next poll.
    if reviews.is_empty() {
        return Err("review-requested search returned no PRs".to_string());
    }
    let assigned = search_prs("--assignee=@me")?;

    // Deduplicate: if a PR appears in both, keep it as Review (higher priority)
    let mut seen: HashSet<(String, u64)> = HashSet::new();
    let mut result: Vec<PullRequest> = Vec::new();

    for raw in reviews {
        let key = (raw.repository.name_with_owner.clone(), raw.number);
        seen.insert(key);
        result.push(PullRequest {
            repository: raw.repository,
            number: raw.number,
            title: raw.title,
            author: raw.author,
            updated_at: raw.updated_at,
            url: raw.url,
            labels: raw.labels,
            kind: PrKind::Review,
        });
    }

    for raw in assigned {
        let key = (raw.repository.name_with_owner.clone(), raw.number);
        if !seen.contains(&key) {
            result.push(PullRequest {
                repository: raw.repository,
                number: raw.number,
                title: raw.title,
                author: raw.author,
                updated_at: raw.updated_at,
                url: raw.url,
                labels: raw.labels,
                kind: PrKind::Assignee,
            });
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_access_fails_when_any_repo_is_unreachable() {
        let repos = vec!["org/ok".to_string(), "org/blocked".to_string()];
        let result = ensure_access(&repos, |r| {
            if r == "org/blocked" { Err("HTTP 403".to_string()) } else { Ok(()) }
        });
        assert_eq!(result, Err("cannot access org/blocked: HTTP 403".to_string()));
    }

    #[test]
    fn ensure_access_passes_when_all_repos_are_reachable() {
        let repos = vec!["org/ok".to_string()];
        assert_eq!(ensure_access(&repos, |_| Ok(())), Ok(()));
    }
}
