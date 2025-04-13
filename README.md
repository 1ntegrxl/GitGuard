# GitGuard 🛡️

> "You ran gitrecon and you want to fix your terrible OPSEC? Don't worry, GitGuard is here for you."

## Overview

GitGuard is a powerful, terminal-friendly Rust tool to sanitize your git history after identifying leaks like unintended email or username exposures.
It's nothing much but I hope it will be helpful for a lot of people :D

## Key Features

- Automatically detects and suggests leaked identities
- Rewrites git history using `git filter-repo`
- SSH connectivity checks for GitHub
- Works on multiple repositories or the current one
- Replace only what's leaked or do a full rewrite (full rewrite not implemented yet but fairly easy to add)

## Requirements

- Rust & Cargo (`rustup`)
- Python 3
- [`git-filter-repo`](https://github.com/newren/git-filter-repo) (installed and in your `$PATH`)

### Install `git-filter-repo`

Ubuntu/Debian:

```bash
$ sudo apt install git-filter-repo
```

Arch:

```bash
$ sudo pacman -S git-filter-repo
```

Manual:

```bash
$ pip install git-filter-repo
```

### One-line install using cargo : 

```bash
$ cargo install --git https://github.com/1ntegrxl/gitguard
```

## Usage

```bash
$ cargo run --release -- \
  --username "your-safe-username" \
  --usual-commit-email "your-safe-email@users.noreply.github.com" \
  --git-projects-path "/path/to/git/folder" \
  --private-key "~/.ssh/id_rsa" \
  --show-leaks/--fix-leaks
  
```
You can either show your leaks or fix them. Fixing them will also show them before doing so. 

## Arguments

| Argument              | Description                                            |
|-----------------------|--------------------------------------------------------|
| `--username`          | The username to replace leaked ones with              |
| `--usual-commit-email`| Safe GitHub noreply email to replace leaked ones      |
| `--git-projects-path` | Path to folder containing Git repositories             |
| `--private-key`       | SSH private key to check GitHub access                 |
| `--show-leaks`        | Only display leaks from previous scans or commits     |
| `--fix-leaks`         | Actually fix the found leaks |

## Workflow

1. Your friend tells you about gitrecon (https://github.com/atiilla/gitrecon) so you go test it and you discover some leaks..

```bash
exegol-HTB Scan # gitrecon -u 1ntegrxl -n 

    ██████╗ ██╗████████╗██████╗ ███████╗ ██████╗ ██████╗ ███╗   ██╗
    ██╔════╝ ██║╚══██╔══╝██╔══██╗██╔════╝██╔════╝██╔═══██╗████╗  ██║
    ██║  ███╗██║   ██║   ██████╔╝█████╗  ██║     ██║   ██║██╔██╗ ██║
    ██║   ██║██║   ██║   ██╔══██╗██╔══╝  ██║     ██║   ██║██║╚██╗██║
    ╚██████╔╝██║   ██║   ██║  ██║███████╗╚██████╗╚██████╔╝██║ ╚████║
     ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝
                                            https://github.com/atiilla
    
Scan all public repositories of 1ntegrxl
Found 4 public repositories
Scanning repository "Wiconsin-cancer-prediction"
Scanning repository "WHarden"
Scanning repository "sat-solver-caml"
Scanning repository "netcaml"
Found the following emails:

┌─────────┬──────────────────────────────────────────────┬────────────┐
│ (index) │ email                                        │ authors    │
├─────────┼──────────────────────────────────────────────┼────────────┤
│ 0       │ '56651736+1ntegrxl@users.noreply.github.com' │ '1ntegrxl' │
└─────────┴──────────────────────────────────────────────┴────────────┘

```
2. You use gitguard to show your leaks in details and fix them.

```bash
exegol-HTB Scan # target/debug/gitguard --username "1ntegrxl" --usual-commit-email "56651736+1ntegrxl@users.noreply.github.com" --private-key "/home/user/.ssh/private_key" --git-projects-path ../repo --fix-leaks



██████╗ ██╗████████╗ ██████╗ ██╗   ██╗ █████╗ ██████╗ ██████╗ 
██╔════╝ ██║╚══██╔══╝██╔════╝ ██║   ██║██╔══██╗██╔══██╗██╔══██╗
██║  ███╗██║   ██║   ██║  ███╗██║   ██║███████║██████╔╝██║  ██║
██║   ██║██║   ██║   ██║   ██║██║   ██║██╔══██║██╔══██╗██║  ██║
╚██████╔╝██║   ██║   ╚██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝
 ╚═════╝ ╚═╝   ╚═╝    ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ 
                                                               
"You ran githunter and you want to fix your terrible 
   OPSEC? Don't worry, gitguard is here for you."

[*] Checking SSH connection...
[*] Executing : ssh -T git@github.com -i /home/user/.ssh/private_key
[+] SSH connection successful.
🔍 Scanning repository test-repo
[-] Mismatch detected: email: 'to@ubefe.com' vs '56651736+1ntegrxl@users.noreply.github.com', name: 'toub' vs '1ntegrxl'
🕵 Found the following potential leaks:

+---------+------------------------------------------+--------------+------+------------+
| (Index) | Commit Hash                              | Email        | Name | Repository |
+---------+------------------------------------------+--------------+------+------------+
| 0       | 41f014731d63d37a9998606970a3f7f2637ce9dd | to@ubefe.com | toub | test-repo  |
+---------+------------------------------------------+--------------+------+------------+
You can fix these leaks by running the tool with --fix-leaks
Starting leak fix process...
[*] Found repository: test-repo
[*] Found leak in repository test-repo: Commit 41f014731d63d37a9998606970a3f7f2637ce9dd | Email: to@ubefe.com | Author: toub
[+] Successfully fixed and pushed history for repository test-repo
🛡 Leaks successfully fixed and pushed!
```
3. Watch your git history get cleaned like a pro 😎

---
