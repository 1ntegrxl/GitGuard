mod git_operations;
mod utils;

use utils::{check_ssh_connection, display_banner, display_table_with_borders, display_message};
use git_operations::{scan_repositories, fix_leaks};

use termcolor::Color;
use clap::{Arg, Command as ClapCommand};
use crossterm::{cursor, execute};
use std::io::{self};
use tokio;

#[tokio::main]
async fn main() {
    display_banner();
    let matches = ClapCommand::new("gitguard")
        .version("1.0")
        .author("1ntegrxl")
        .about("GitHub Repository Commit and leak Cleaner - OPSEC tool")
        .arg(Arg::new("username")
            .long("username")
            .value_parser(clap::value_parser!(String))
            .help("Your GitHub username")
            .required(true)) // Make username a required argument
        .arg(Arg::new("usual-commit-email")
            .long("usual-commit-email")
            .value_parser(clap::value_parser!(String))
            .help("The usual commit email")
            .required(true)) // Make usual-commit-email a required argument
        .arg(Arg::new("private-key")
            .long("private-key")
            .value_parser(clap::value_parser!(String))
            .help("The private SSH key for GitHub"))
        .arg(Arg::new("git-projects-path")
            .long("git-projects-path")
            .value_parser(clap::value_parser!(String))
            .help("Path to Git projects (default: ./repos)"))
        .arg(Arg::new("show-leaks")
            .long("show-leaks")
            .action(clap::ArgAction::SetTrue)
            .help("Show leaks without fixing them"))
        .arg(Arg::new("fix-leaks")
            .long("fix-leaks")
            .action(clap::ArgAction::SetTrue)
            .help("Fix leaks by rewriting history using git filter-repo"))
        .get_matches();

    // Show help if no arguments or --help is provided
    if matches.args_present() == false || matches.get_one::<String>("help").is_some() {
        ClapCommand::new("gitguard").print_help().unwrap();
        return;
    }

    // Read arguments
    let username = matches
        .get_one::<String>("username")
        .unwrap()
        .to_string();
    let usual_commit_email = matches
        .get_one::<String>("usual-commit-email")
        .unwrap()
        .to_string();
    let git_projects_path = matches
        .get_one::<String>("git-projects-path")
        .unwrap_or(&"./repos".to_string())
        .to_string();
    let private_key = matches.get_one::<String>("private-key");

    // Check if --fix-leaks is set but --private-key is not provided
    if matches.get_flag("fix-leaks") && private_key.is_none() {
        display_message(
            "[-] Make sure you provide your private key to check SSH connection when using --fix-leaks.",
            Color::Red,
        );
        return; 
    }

    let mut stdout = io::stdout();

    // Check SSH connection if private-key is provided
    if let Some(private_key) = private_key {
        println!("[*] Checking SSH connection...");
        if !check_ssh_connection(private_key).await {
            display_message("[!] SSH connection failed", Color::Red);
            return;
        }
    }

    // Call scan_repositories function to get leaks
    let leaks = scan_repositories(&git_projects_path, &username, &usual_commit_email).await;

    if leaks.is_empty() {
        display_message("🛡️ No leaks found!", Color::Green);
    } else {
        display_message("🕵️ Found the following potential leaks:\n", Color::Yellow);
        display_table_with_borders(leaks.clone());
        display_message("You can fix these leaks by running the tool with --fix-leaks", Color::White);
        
        // Fix leaks if the flag is set
        if matches.get_flag("fix-leaks") {
            display_message("Starting leak fix process...", Color::Blue);
            fix_leaks(
                &username,
                &usual_commit_email,
                &git_projects_path,
                &leaks,
            ).await;
            display_message("🛡️ Leaks successfully fixed and pushed!", Color::Green);
        }
    }

    execute!(stdout, cursor::Show).unwrap();
}




