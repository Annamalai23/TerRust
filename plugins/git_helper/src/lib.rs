pub fn get_git_branch() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            Some(branch)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_git_status() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout);
        let changed: Vec<&str> = status.lines().collect();
        if changed.is_empty() {
            Some("clean".to_string())
        } else {
            Some(format!("{} changed", changed.len()))
        }
    } else {
        None
    }
}

pub fn is_git_repo() -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_git_prompt() -> Option<String> {
    if !is_git_repo() {
        return None;
    }
    let branch = get_git_branch()?;
    let status = get_git_status().unwrap_or_default();
    if status == "clean" {
        Some(format!("({})", branch))
    } else {
        Some(format!("({} {})", branch, status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_repo_in_test_env() {
        let result = is_git_repo();
        assert!(!result);
    }

    #[test]
    fn test_get_git_branch_outside_repo() {
        let branch = get_git_branch();
        assert!(branch.is_none());
    }

    #[test]
    fn test_get_git_status_outside_repo() {
        let status = get_git_status();
        assert!(status.is_none());
    }

    #[test]
    fn test_get_git_prompt_outside_repo() {
        let prompt = get_git_prompt();
        assert!(prompt.is_none());
    }
}
