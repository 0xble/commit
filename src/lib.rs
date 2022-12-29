use colored::*;
use reqwest::blocking::Client;
use serde_json::json;
use spinners::{Spinner, Spinners};
use std::process;

pub fn generate_commit_message(
    commit_messages: &str,
    mut diff: String,
    api_key: &str,
    max_tokens: usize,
    negative_matches: &str,
) -> (String, String) {
    let client = Client::new();

    let mut data = None;

    // Loop at most twice to retry with summary of changes if diff is too long
    for _ in 0..2 {
        let mut spinner =
            Spinner::new(Spinners::BouncingBar, "Generating commit message...".into());

        // Create prompt
        let mut prompt = String::new();

        if !commit_messages.is_empty() {
            prompt = format!("Using the same format, convention, and style of these examples:\n{commit_messages}\n\n");
        };

        prompt = match negative_matches.len() {
            0 => format!("{prompt}Write a commit message no longer than seventy-two characters describing the changes, ignoring todo comments:\n{diff}\n\nShow me just the commit message."),
            _ => format!("{prompt}Write a commit message no longer than seventy-two characters describing the changes, ignoring todo comments:\n{diff}\n\nAvoid generating these messages:\n{negative_matches}\n\nShow me just the commit message."),
        };

        // Send request to OpenAI API
        let response = client
            .post("https://api.openai.com/v1/completions")
            .json(&json!({
                "top_p": 1,
                "temperature": 0,
                "max_tokens": max_tokens,
                "presence_penalty": 0,
                "frequency_penalty": 0,
                "model": "text-davinci-003",
                "prompt": prompt,
            }))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .unwrap_or_else(|_| {
                spinner.stop_and_persist(
                    "✖".red().to_string().as_str(),
                    "Failed to get a response. Have you set the OPENAI_API_KEY variable?"
                        .red()
                        .to_string(),
                );
                std::process::exit(1);
            });

        data = Some(response.json::<serde_json::Value>().unwrap());

        // Check for error
        let error = &data.as_ref().unwrap()["error"]["message"];
        if error.is_string() && error.to_string().contains("Please reduce your prompt") {
            // If the diff is too long, generate message from stat summary of diff instead
            let new_diff = process::Command::new("git")
                .arg("diff")
                .arg("--staged")
                .arg("--stat")
                .arg("--summary")
                .output()
                .expect("Failed to execute `git diff`");

            spinner.stop_with_message(
                "Exceeds max tokens, using summary of changes instead..."
                    .yellow()
                    .to_string(),
            );

            diff = String::from_utf8(new_diff.stdout).unwrap();
        } else {
            spinner.stop_and_persist(
                "✔".green().to_string().as_str(),
                "Got commit message!".green().to_string(),
            );

            break;
        }
    }

    // Get commit message from response
    let commit;
    match data {
        Some(value) => {
            commit = value["choices"][0]["text"]
                .as_str()
                .unwrap_or_else(|| {
                    if value["error"]["message"].is_string() {
                        println!(
                            "{}",
                            format!("{}", value["error"]["message"].to_string()).red()
                        );
                        std::process::exit(1);
                    } else {
                        println!("{}", "Nothing returned from GPT-3.".red());
                        std::process::exit(1);
                    }
                })
                .trim()
                .trim_start_matches("\"")
                .trim_end_matches("\"")
                .to_string();
        }
        None => {
            println!("{}", "Failed to generate commit message.".red());
            std::process::exit(1);
        }
    }

    if commit.is_empty() {
        println!("{}", "Nothing returned from GPT-3.".red());
        std::process::exit(1);
    }

    (commit, diff)
}
