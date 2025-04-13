use crate::utils::display_message;
use tokio::fs as tokio_fs;
use termcolor::Color;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::process::Stdio;



pub async fn scan_repositories(git_projects_path: &str, username: &str, usual_commit_email: &str) -> Vec<(String, String, String, String)> {
    let mut leaks = Vec::new();
    let path = tokio_fs::canonicalize(git_projects_path)
        .await
        .unwrap_or_else(|_| {
            eprintln!("[!] Invalid path provided: {}", git_projects_path);
            std::process::exit(1);
        });

    if !path.exists() || !path.is_dir() {
        eprintln!("[!] Directory not found: {}", git_projects_path);
        return leaks;
    }

    let mut entries = tokio_fs::read_dir(&path)
        .await
        .expect("Failed to read directory");

    while let Some(entry) = entries.next_entry().await.expect("Failed to read entry") {
        let path = entry.path();

        if path.is_dir() && path.join(".git").exists() {
            let repo_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            display_message(&format!("🔍 Scanning repository {}", repo_name), Color::Yellow);

            // Get git commit logs for each repository
            let output = Command::new("git")
                .arg("log")
                .arg("--all")
                .arg("--pretty=format:%H|%ae|%an|%s")
                .arg("--no-merges")
                .current_dir(&path)
                .output()
                .expect("Failed to execute git log");

            if output.status.success() {
                let logs = String::from_utf8_lossy(&output.stdout);
                for log in logs.split("\n") {
                    let parts: Vec<&str> = log.split('|').collect();
                    if parts.len() == 4 {
                        let commit_hash = parts[0];
                        let email = parts[1];
                        let name = parts[2];
                        // Check for leaks in the commit email and username
                        if email.trim().to_lowercase() != usual_commit_email.trim().to_lowercase()
                        || name.trim().to_lowercase() != username.trim().to_lowercase() {
                            println!(
                                "[-] Mismatch detected: email: '{}' vs '{}', name: '{}' vs '{}'",
                                email.trim().to_lowercase(),
                                usual_commit_email.trim().to_lowercase(),
                                name.trim().to_lowercase(),
                                username.trim().to_lowercase()
                            );
                        
                            leaks.push((
                                commit_hash.to_string(),
                                email.to_string(),
                                name.to_string(),
                                repo_name.clone(),
                            ));
                        }
                    }
                }
            }
        }
    }
    leaks
}

pub async fn fix_leaks(
    username: &str,
    usual_commit_email: &str,
    git_projects_path: &str,
    leaks: &[(String, String, String, String)],
) {
    let repo_dir = Path::new(git_projects_path);

    if !repo_dir.exists() {
        display_message(
            &format!("[!] Repository path does not exist: {}", git_projects_path),
            Color::Red,
        );
        return;
    }

    let entries = match fs::read_dir(repo_dir) {
        Ok(entries) => entries,
        Err(_) => {
            display_message(
                &format!("[!] Failed to read directory: {}", git_projects_path),
                Color::Red,
            );
            return;
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let repo_path = entry.path();
        if repo_path.is_dir() && repo_path.join(".git").exists() {
            let repo_name = repo_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            display_message(
                &format!("[*] Found repository: {}", repo_name),
                Color::Blue,
            );

            let mut filter_script = String::new();

            // Collect commits that need to be rewritten
            for leak in leaks {
                let (commit_hash, email, name, _) = (
                    leak.0.clone(),
                    leak.1.clone(),
                    leak.2.clone(),
                    leak.3.clone(),
                );
                display_message(
                    &format!(
                        "[*] Found leak in repository {}: Commit {} | Email: {} | Author: {}",
                        repo_name, commit_hash, email, name
                    ),
                    Color::Red,
                );

                // Update both author and committer information
                filter_script.push_str(&format!(
                    r#"
                    if commit.original_id == b"{}":
                        commit.author_name = b"{}"
                        commit.author_email = b"{}"
                        commit.committer_name = b"{}"
                        commit.committer_email = b"{}"
                    "#,
                    commit_hash, username, usual_commit_email, username, usual_commit_email
                ));
            }

            let output = Command::new("git")
                .arg("filter-repo")
                .arg("--commit-callback")
                .arg(&filter_script)
                .arg("--force")
                .current_dir(&repo_path)
                .output()
                .expect("[-] Failed to execute git filter-repo");

            if !output.status.success() {
                display_message(
                    &format!("[-] Failed to rewrite commits in repository {}", repo_name),
                    Color::Red,
                );
                continue;
            }

            let default_branch = get_default_branch(&repo_path);

            if default_branch.is_empty() {
                display_message(
                    &format!(
                        "[!] Failed to detect the default branch in repository {}",
                        repo_name
                    ),
                    Color::Red,
                );
                continue;
            }

            let folder_name = repo_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let _remove_output = Command::new("git")
                .arg("remote")
                .arg("remove")
                .arg("origin")
                .stderr(Stdio::null()) // Suppress error if origin doesn't exist
                .current_dir(&repo_path)
                .output()
                .expect("[-] Failed to remove remote origin");

            let _add_output = Command::new("git")
                .arg("remote")
                .arg("add")
                .arg("origin")
                .arg(format!("git@github.com:{}/{}.git", username, folder_name))
                .current_dir(&repo_path)
                .output()
                .expect("[-] Failed to add remote origin");

            let push_output = Command::new("git")
                .arg("push")
                .arg("--set-upstream")
                .arg("origin")
                .arg(&default_branch)
                .arg("--force")
                .current_dir(&repo_path)
                .output()
                .expect("[-] Failed to push rewritten history");

            if push_output.status.success() {
                display_message(
                    &format!(
                        "[+] Successfully fixed and pushed history for repository {}",
                        folder_name
                    ),
                    Color::Green,
                );
            } else {
                display_message(
                    &format!("[!] Failed to push to repository {}", folder_name),
                    Color::Red,
                );
            }
        } else {
            display_message(
                &format!(
                    "[!] Skipping non-repository directory: {}",
                    repo_path.display()
                ),
                Color::Yellow,
            );
        }
    }
}
// get the default branch (main or master)
fn get_default_branch(repo_dir: &Path) -> String {
    let output = Command::new("git")
        .arg("symbolic-ref")
        .arg("--short")
        .arg("HEAD")
        .current_dir(repo_dir)
        .output()
        .expect("[-] Failed to get current branch");

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "main".to_string() // Default to 'main' if symbolic ref fails
    }
}
