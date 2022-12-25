use bat::PrettyPrinter;
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use colored::*;
use commit::generate_commit_message;
use dialoguer::{theme::ColorfulTheme, Editor, Select};
use std::{env, process};
use words_count::count;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[clap(help = "Files to stage and commit")]
    files: Vec<String>,
    #[clap(
        short,
        long,
        default_value = "50",
        help = "Number of commits in history to use generating message"
    )]
    commits: usize,
    #[clap(
        short = 't',
        long,
        default_value = "2000",
        help = "Maximum number of tokens to use generating message"
    )]
    max_tokens: usize,
    #[clap(
        long,
        help = "Copy the commit message to clipboard instead of committing"
    )]
    copy: bool,
    #[clap(long, help = "Don't commit, just print the commit message to stdout")]
    no_commit: bool,
    #[clap(short, long, help = "Answert \"Yes\" to prompts")]
    yes: bool,
}

fn main() {
    // Parse the command line arguments
    let Cli {
        files,
        commits,
        max_tokens,
        copy,
        no_commit,
        yes,
    } = Cli::parse();

    // Check if the OpenAI API key is set
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("{}", "This program requires an OpenAI API key to run. Please set the OPENAI_API_KEY environment variable.".red());
        std::process::exit(1);
    });

    // Stage the files if any are provided
    if !files.is_empty() {
        process::Command::new("git")
            .arg("add")
            .args(&files)
            .output()
            .expect("Failed to execute `git add`");
    }

    // Get the diff of the staged files
    let diff = process::Command::new("git")
        .arg("diff")
        .arg("--staged")
        .arg("--minimal")
        .output()
        .expect("Failed to execute `git diff`");
    let mut diff = String::from_utf8(diff.stdout).unwrap();

    let count = count(&diff);
    if count.characters - count.whitespaces == 0 {
        println!(
            "{}",
            "Nothing to commit. Did you stage your changes with \"git add\"?".red()
        );
        std::process::exit(1);
    }

    // Get the commit messages from the last n commits
    let commit_messages = process::Command::new("git")
        .arg("log")
        .arg(format!("-{}", commits))
        .arg("--pretty=format:%s")
        .output()
        .expect("Failed to execute `git log`");
    let mut commit_messages = String::from_utf8(commit_messages.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("Merge"))
        .map(|line| format!("- {line}"))
        .collect::<Vec<String>>()
        .join("\n");
    // Remove PR numbers from squashed commit messages
    let re = regex::Regex::new(r"\(#\d+\)\n").unwrap();
    commit_messages = re.replace_all(&commit_messages, "\n").to_string();

    // Generate the commit message using GPT-3
    let mut commit;
    let mut negative_matches = vec![];
    loop {
        let prompt = match (copy, no_commit) {
            (true, false) | (true, true) => "Copy commit message to clipboard?",
            (false, true) => "Print commit message to stdout?",
            (false, false) => "Commit changes with message?",
        };

        (commit, diff) = generate_commit_message(
            &commit_messages,
            diff.clone(),
            &api_key,
            max_tokens,
            negative_matches.join("\n").as_str(),
        );

        // If user has provided the --yes flag, confirm without prompting
        if yes {
            break;
        }

        // Print commit message
        PrettyPrinter::new()
            .input_from_bytes(commit.as_bytes())
            .grid(true)
            .colored_output(false)
            .print()
            .unwrap();

        let user_option = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .item("Yes")
            .item("No")
            .item("Edit")
            .item("Redo")
            .interact()
            .unwrap();

        // Handle user selection
        match user_option {
            // Proceed if user selects "Yes"
            0 => break,
            // Quit if user selects "No"
            1 => std::process::exit(0),
            // Edit the commit message if user selects "Edit"
            2 => {
                if let Some(new_commit) = Editor::new().edit(&commit).unwrap() {
                    if new_commit.is_empty() {
                        println!("{}", "Commit message cannot be empty.".red());
                        std::process::exit(1);
                    }
                    commit = new_commit;
                    break;
                } else {
                    std::process::exit(0);
                }
            }
            // Redo the commit message if user selects "Redo"
            3 => {
                negative_matches.push(format!("- {}", commit));
            }
            // Unrecognized selection
            _ => {
                println!("{}", "Unrecognized selection.".red());
                std::process::exit(1);
            }
        }
    }
    if copy {
        // Copy the header to clipboard.
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(commit.clone()).unwrap();
    } else if no_commit {
        // Print the commit message to stdout
        println!("{}", commit);
    } else {
        // Commit changes with generated commit message
        let commit = process::Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit)
            .output()
            .expect("Failed to execute `git commit`");
        let commit = String::from_utf8(commit.stdout).unwrap();
        let commit = commit.trim();
        println!("{}", commit);
    }
}
