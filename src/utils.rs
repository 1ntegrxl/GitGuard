use tokio;
use std::io::Write;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use prettytable::{Table, Row, Cell};
use tokio::process::Command as TokioCommand;
use termcolor::{Color, ColorSpec, ColorChoice, StandardStream, WriteColor};


pub fn display_banner() {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(Color::Green));
    stdout.set_color(&color_spec).unwrap();
    
    println!("\n\n██████╗ ██╗████████╗ ██████╗ ██╗   ██╗ █████╗ ██████╗ ██████╗ ");
    println!("██╔════╝ ██║╚══██╔══╝██╔════╝ ██║   ██║██╔══██╗██╔══██╗██╔══██╗");
    println!("██║  ███╗██║   ██║   ██║  ███╗██║   ██║███████║██████╔╝██║  ██║");
    println!("██║   ██║██║   ██║   ██║   ██║██║   ██║██╔══██║██╔══██╗██║  ██║");
    println!("╚██████╔╝██║   ██║   ╚██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝");
    println!(" ╚═════╝ ╚═╝   ╚═╝    ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ");
    println!("                                                               ");
    
    color_spec.set_fg(None);
    stdout.set_color(&color_spec).unwrap();
    println!("\"You ran githunter and you want to fix your terrible ");
    println!("   OPSEC? Don't worry, gitguard is here for you.\"\n");
}

// Asynchronous function to check SSH connection to GitHub
pub async fn check_ssh_connection(private_key: &str) -> bool {
    let mut command = TokioCommand::new("ssh");
    command
        .arg("-T")
        .arg("git@github.com")
        .arg("-i")
        .arg(private_key)
        .stderr(Stdio::piped()) 
        .stdout(Stdio::piped());

    println!("[*] Executing : ssh -T git@github.com -i {}", private_key);

    let mut child = command.spawn().expect("[-] Failed to start SSH command");

    let mut stdout = child.stdout.take().expect("[-] Failed to capture stdout");
    let mut stderr = child.stderr.take().expect("[-] Failed to capture stderr");
    let mut buffer = vec![0u8; 1024];

    while let Ok(bytes_read) = stdout.read(&mut buffer).await {
        if bytes_read == 0 {
            break;
        }
        let stdout_output = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", stdout_output); 
    }
    while let Ok(bytes_read) = stderr.read(&mut buffer).await {
        if bytes_read == 0 {
            break; // End of stderr output
        }
        let stderr_output = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", stderr_output);
    }

    let output = child.wait().await.expect("Failed to wait on SSH command");

    if output.success() {
        println!("[+] SSH connection successful.");
        return true;
    } else {
        let stdout_str = String::from_utf8_lossy(&buffer);
        if stdout_str.contains("Hi") {
            println!("[+] SSH connection successful.");
            return true;
        } else {
            println!("[!] SSH connection failed.");
            return false;
        }
    }
}

pub fn display_message(message: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(color));
    stdout.set_color(&color_spec).unwrap();
    write!(&mut stdout, "{}\n", message).unwrap();
    stdout.reset().unwrap();
}


pub fn display_table_with_borders(leaks: Vec<(String, String, String, String)>) {
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("(Index)"),
        Cell::new("Commit Hash"),
        Cell::new("Email"),
        Cell::new("Name"),
        Cell::new("Repository"),
    ]));

    for (index, leak) in leaks.iter().enumerate() {
        let (commit_hash, email, name, repo_name) = (leak.0.clone(), leak.1.clone(), leak.2.clone(), leak.3.clone());
        table.add_row(Row::new(vec![
            Cell::new(&index.to_string()),
            Cell::new(&commit_hash),
            Cell::new(&email),
            Cell::new(&name),
            Cell::new(&repo_name),
        ]));
    }

    table.printstd();
}

