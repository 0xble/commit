# Commit

![commit](https://user-images.githubusercontent.com/20782088/209482689-4b4b1a2c-9ae4-4ed0-ac62-ba0dceef5c44.png)

Generate commit messages using GPT-3 based on your changes and commit history.

## Install

You need Rust and Cargo installed on your machine. See the installation guide
[here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Then clone the repo and install the CLI globally like this:

```sh
cargo install --path .
```

## Usage

`commit` uses [GPT-3](https://beta.openai.com/). To use it, you'll need to grab an API key from [your dashboard](https://beta.openai.com/), and save it to `OPENAI_API_KEY` as follows (you can also save it in your bash/zsh profile for persistance between sessions).

```bash
export OPENAI_API_KEY='sk-XXXXXXXX'
```

Once you have configured your environment, run `commit` in any Git repository with staged changes.

To get a full overview of all available options, run `commit --help`

```sh
$ commit --help
Generate commit messages using GPT-3 based on your changes and commit history.

Usage: commit [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Files to stage and commit

Options:
  -c, --commits <COMMITS>        Number of commits in history to use generating message [default: 50]
  -t, --max-tokens <MAX_TOKENS>  Maximum number of tokens to use generating message [default: 2000]
      --copy                     Copy the commit message to clipboard instead of committing
      --no-commit                Don't commit, just print the commit message to stdout
  -y, --yes                      Answert "Yes" to prompts
  -h, --help                     Print help information
  -V, --version                  Print version information
```
