use std::io;
use std::process::ExitCode;

use crate::types::Source;

pub fn run() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ai-usage: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> io::Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    if args.help {
        print_help();
        return Ok(());
    }

    if args.init_config {
        if args.all || !args.sources.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--init-config cannot be combined with source flags or --all",
            ));
        }

        let path = crate::config::init()?;
        println!("Created config: {}", path.display());
        return Ok(());
    }

    let sources = select_sources(args)?;
    let report = crate::get_limits::get_limits(&sources)?;

    for summary in report.summaries {
        println!("{summary}");
    }

    if !report.stderr.trim().is_empty() {
        eprint!("{}", report.stderr);
    }

    Ok(())
}

struct CliArgs {
    help: bool,
    init_config: bool,
    all: bool,
    sources: Vec<Source>,
}

fn parse_args(args: impl Iterator<Item = String>) -> io::Result<CliArgs> {
    let mut parsed = CliArgs {
        help: false,
        init_config: false,
        all: false,
        sources: Vec::new(),
    };
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                parsed.help = true;
            }
            "--init-config" => {
                parsed.init_config = true;
            }
            "-a" | "--all" => {
                parsed.all = true;
            }
            "--codex-cli" => {
                parsed.sources.push(Source::CodexCli);
            }
            "--claude-cli" => {
                parsed.sources.push(Source::ClaudeCli);
            }
            "--cursor-api2" => {
                parsed.sources.push(Source::CursorApi2);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument `{arg}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn print_help() {
    println!(
        "\
ai-usage

Usage:
  ai-usage [OPTIONS]

Options:
  --help, -h      Show this help
  --init-config   Create the user config file if it does not exist
  --all, -a       Query all current sources, ignoring config defaults
  --codex-cli     Query Codex through the Codex CLI
  --claude-cli    Query Claude through the Claude CLI
  --cursor-api2   Query Cursor through api2.cursor.sh

Config:
  ~/.config/ai-usage/config.toml

  default_sources = [\"codex_cli\", \"claude_cli\", \"cursor_api2\"]
"
    );
}

fn select_sources(args: CliArgs) -> io::Result<Vec<Source>> {
    if args.all && !args.sources.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--all cannot be combined with source flags",
        ));
    }

    if args.all {
        return Ok(Source::ALL.to_vec());
    }

    if !args.sources.is_empty() {
        return Ok(args.sources);
    }

    let Some(config) = crate::config::load()? else {
        return Ok(Source::ALL.to_vec());
    };

    if config.default_sources.is_empty() {
        Ok(Source::ALL.to_vec())
    } else {
        Ok(config.default_sources)
    }
}
