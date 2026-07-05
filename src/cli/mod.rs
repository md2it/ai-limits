use std::collections::HashSet;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::get_limits::SourcePlan;
use crate::infra::loader::{
    loader_show_delay, loader_tick, LoaderView, TerminalStatus, TerminalUi,
};
use crate::notifications::LimitNotificationKind;
use crate::presentation::{
    format_raw_output, format_structured_output, limits_block, usage_block, ColorConfig,
    ProviderBlock,
};
use crate::types::{Source, SourceReport};

pub fn run() -> ExitCode {
    match run_cli() {
        Ok(status) => match status {
            TerminalStatus::Done | TerminalStatus::Part => ExitCode::SUCCESS,
            TerminalStatus::Fail => ExitCode::FAILURE,
        },
        Err(error) => {
            let mut ui = TerminalUi::new();
            let _ = ui.print_top();
            println!("ai-limits: {error}");
            let _ = ui.print_bottom(TerminalStatus::Fail);
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> io::Result<TerminalStatus> {
    let args = parse_args(std::env::args().skip(1))?;

    if args.help {
        let mut ui = TerminalUi::new();
        ui.print_top()?;
        print_help();
        ui.print_bottom(TerminalStatus::Done)?;
        return Ok(TerminalStatus::Done);
    }

    if args.init_config {
        if args.all
            || args.best
            || !args.sources.is_empty()
            || args.watch.is_some()
            || args.test_notification.is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--init-config cannot be combined with source flags, --all, --best, --watch, or --test-notification",
            ));
        }

        let status = run_init_config()?;
        return Ok(status);
    }

    if let Some(kind) = args.test_notification {
        if args.all || args.best || !args.sources.is_empty() || args.watch.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--test-notification cannot be combined with source flags, --all, --best, or --watch",
            ));
        }

        crate::notifications::notify_test(kind)?;
        return Ok(TerminalStatus::Done);
    }

    let config = crate::config::load()?;
    let watch_interval = resolve_watch_interval(&args, config.as_ref());
    let output_mode = args.output_mode;
    let plan = resolve_source_plan(args, config)?;
    let mut sent_notifications = HashSet::new();

    if let Some(interval) = watch_interval {
        loop {
            run_once(&plan, output_mode, &mut sent_notifications)?;
            thread::sleep(interval);
        }
    }

    let status = run_once(&plan, output_mode, &mut sent_notifications)?;

    Ok(status)
}

fn run_once(
    plan: &[SourcePlan],
    output_mode: OutputMode,
    sent_notifications: &mut HashSet<String>,
) -> io::Result<TerminalStatus> {
    let mut ui = TerminalUi::new();
    ui.print_top()?;
    let status = run_sources_with_terminal_ui(&mut ui, plan, output_mode, sent_notifications)?;
    ui.print_bottom(status)?;
    Ok(status)
}

struct RunningSource {
    label: &'static str,
    started_at: Instant,
    loader_shown: bool,
    loader_frame: usize,
}

struct SourceEvent {
    label: &'static str,
    result: io::Result<SourceReport>,
}

fn run_init_config() -> io::Result<TerminalStatus> {
    let path = crate::config::config_path()?;
    let mut ui = TerminalUi::new();
    ui.print_top()?;

    if path.exists() {
        if !prompt_overwrite(&path, &mut io::stdin().lock(), &mut io::stdout())? {
            println!("Config init cancelled.");
            ui.print_bottom(TerminalStatus::Done)?;
            return Ok(TerminalStatus::Done);
        }

        crate::config::write_default(&path)?;
        println!("Overwritten config: {}", path.display());
    } else {
        crate::config::write_default(&path)?;
        println!("Created config: {}", path.display());
    }

    ui.print_bottom(TerminalStatus::Done)?;
    Ok(TerminalStatus::Done)
}

fn prompt_overwrite(
    path: &Path,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> io::Result<bool> {
    if !io::stdin().is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("config already exists: {}", path.display()),
        ));
    }

    read_overwrite_confirmation(path, reader, writer)
}

fn read_overwrite_confirmation(
    path: &Path,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> io::Result<bool> {
    write!(
        writer,
        "Config already exists at {}. Overwrite? [y/n] ",
        path.display()
    )?;
    writer.flush()?;

    let mut answer = String::new();
    reader.read_line(&mut answer)?;

    let trimmed = answer.trim();
    Ok(trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes"))
}

fn run_sources_with_terminal_ui(
    ui: &mut TerminalUi,
    plan: &[SourcePlan],
    output_mode: OutputMode,
    sent_notifications: &mut HashSet<String>,
) -> io::Result<TerminalStatus> {
    if plan.is_empty() {
        return Ok(TerminalStatus::Fail);
    }

    let color = ColorConfig::from_env(io::stdout().is_terminal());
    let (sender, receiver) = mpsc::channel::<SourceEvent>();
    let mut running = Vec::new();

    for target in plan {
        let target = *target;
        let label = target.label();
        let sender = sender.clone();
        running.push(RunningSource {
            label,
            started_at: Instant::now(),
            loader_shown: false,
            loader_frame: 0,
        });
        thread::spawn(move || {
            let result = crate::get_limits::get_source_plan_limits(target);
            let _ = sender.send(SourceEvent { label, result });
        });
    }
    drop(sender);

    let mut successes = 0_usize;
    let mut failures = 0_usize;
    let mut stderr = String::new();

    while !running.is_empty() {
        render_running_loaders(ui, &mut running)?;

        match receiver.recv_timeout(loader_tick()) {
            Ok(event) => {
                if let Some(index) = running
                    .iter()
                    .position(|running| running.label == event.label)
                {
                    running.remove(index);
                }

                match event.result {
                    Ok(report) => {
                        successes += 1;
                        stderr.push_str(&report.data.stderr);
                        print_source_report(ui, &report, output_mode, &color)?;
                        crate::notifications::send_for_report(&report, sent_notifications);
                    }
                    Err(error) => {
                        failures += 1;
                        let block = failed_source_block(event.label, &error.to_string());
                        ui.print_provider_block(&block.provider_label, &block.body)?;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    ui.finish_loaders()?;

    if !stderr.trim().is_empty() {
        eprint!("{stderr}");
    }

    Ok(match (successes, failures) {
        (_, 0) if successes > 0 => TerminalStatus::Done,
        (0, _) => TerminalStatus::Fail,
        _ => TerminalStatus::Part,
    })
}

fn print_source_report(
    ui: &mut TerminalUi,
    report: &SourceReport,
    output_mode: OutputMode,
    color: &ColorConfig,
) -> io::Result<()> {
    let block = match output_mode {
        OutputMode::Limits => limits_block(&report.data.structured, color),
        OutputMode::Usage => usage_block(&report.data.structured),
        OutputMode::Raw => ProviderBlock {
            provider_label: report.data.structured.provider.to_ascii_uppercase(),
            body: format_raw_output(&report.data),
        },
        OutputMode::Structured => ProviderBlock {
            provider_label: report.data.structured.provider.to_ascii_uppercase(),
            body: format_structured_output(&report.data),
        },
    };

    ui.print_provider_block(&block.provider_label, &block.body)
}

fn failed_source_block(label: &str, error: &str) -> ProviderBlock {
    let provider = match label {
        "codex" | "codex-local" | "codex-cli" => "CODEX",
        "claude" | "claude-statusline" | "claude-cli" | "claude-local" => "CLAUDE",
        "cursor" | "cursor-api2" => "CURSOR",
        _ => "AI LIMITS",
    };

    ProviderBlock {
        provider_label: provider.to_string(),
        body: format!("Unavailable: {error}\nSource {label}: unknown"),
    }
}

fn render_running_loaders(ui: &mut TerminalUi, running: &mut [RunningSource]) -> io::Result<()> {
    for running in running.iter_mut() {
        if running.started_at.elapsed() >= loader_show_delay() {
            running.loader_shown = true;
        }
        if running.loader_shown {
            running.loader_frame = running.loader_frame.wrapping_add(1);
        }
    }

    let loaders = running
        .iter()
        .filter(|running| running.loader_shown)
        .map(|running| LoaderView {
            label: running.label,
            frame: running.loader_frame,
        })
        .collect::<Vec<_>>();

    if loaders.is_empty() {
        return Ok(());
    }

    ui.render_loaders(&loaders)
}

struct CliArgs {
    help: bool,
    init_config: bool,
    all: bool,
    best: bool,
    watch: Option<Option<Duration>>,
    output_mode: OutputMode,
    sources: Vec<Source>,
    test_notification: Option<LimitNotificationKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Limits,
    Usage,
    Raw,
    Structured,
}

fn parse_args(args: impl Iterator<Item = String>) -> io::Result<CliArgs> {
    let mut parsed = CliArgs {
        help: false,
        init_config: false,
        all: false,
        best: false,
        watch: None,
        output_mode: OutputMode::Limits,
        sources: Vec::new(),
        test_notification: None,
    };
    let mut args = args.peekable();
    let mut output_mode = None;

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
            "-b" | "--best" => {
                parsed.best = true;
            }
            "-w" | "--watch" => {
                parsed.watch = Some(None);
            }
            "--usage" => {
                if output_mode.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--usage cannot be combined with --raw or --structured",
                    ));
                }
                output_mode = Some(OutputMode::Usage);
            }
            "-r" | "--raw" => {
                if output_mode.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--raw cannot be combined with other output flags",
                    ));
                }
                output_mode = Some(OutputMode::Raw);
            }
            "-s" | "--structured" => {
                if output_mode.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--structured cannot be combined with other output flags",
                    ));
                }
                output_mode = Some(OutputMode::Structured);
            }
            "--codex-local" => {
                parsed.sources.push(Source::CodexLocal);
            }
            "--codex-cli" => {
                parsed.sources.push(Source::CodexCli);
            }
            "--claude-cli" => {
                parsed.sources.push(Source::ClaudeCli);
            }
            "--claude-statusline" => {
                parsed.sources.push(Source::ClaudeStatusline);
            }
            "--claude-local" => {
                parsed.sources.push(Source::ClaudeLocal);
            }
            "--cursor-api2" => {
                parsed.sources.push(Source::CursorApi2);
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--watch=") {
                    parsed.watch = Some(Some(parse_watch_interval_arg(value)?));
                    continue;
                }
                if let Some(value) = arg.strip_prefix("-w=") {
                    parsed.watch = Some(Some(parse_watch_interval_arg(value)?));
                    continue;
                }
                if let Some(value) = arg.strip_prefix("--test-notification=") {
                    parsed.test_notification =
                        Some(LimitNotificationKind::parse(value).map_err(|error| {
                            io::Error::new(
                                io::ErrorKind::InvalidInput,
                                format!("invalid --test-notification value: {error}"),
                            )
                        })?);
                    continue;
                }

                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument `{arg}`"),
                ));
            }
        }
    }

    if let Some(output_mode) = output_mode {
        parsed.output_mode = output_mode;
    }

    Ok(parsed)
}

fn parse_watch_interval_arg(value: &str) -> io::Result<Duration> {
    crate::config::parse_duration(value).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid --watch interval: {error}"),
        )
    })
}

fn print_help() {
    println!(
        "\
Usage:
  ai-limits [OPTIONS]

Options:
  --help, -h       Show this help
  --init-config    Create / overwrite the user config file
  --all, -a        Query all current sources, ignoring config defaults
  --best, -b       Query best available source per provider
  --watch, -w      Repeat the query on an interval
  --usage          Show user-facing usage summary
  --raw, -r        Return raw source data
  --structured, -s Return structured source data
  --test-notification=<75|50|25|10>
                  Send a test system notification and exit

Technical source options:
  --codex-local       Query Codex from local session JSONL files
  --codex-cli         Query Codex through the Codex CLI
  --claude-statusline Query Claude live limits from statusline cache/stdin
  --claude-cli        Query Claude through the Claude CLI
  --claude-local      Query Claude from local transcript JSONL files
  --cursor-api2       Query Cursor through api2.cursor.sh

Examples:
  ai-limits --all
  ai-limits --best
  ai-limits --watch
  ai-limits --watch=10m
  ai-limits --all --usage
  ai-limits --all --raw
  ai-limits --all --structured
  ai-limits --test-notification=75

Config:
  ~/.config/ai-limits/config.toml

  # Leave empty to use the built-in fast free provider chains.
  default_sources = []
  watch_interval = \"5m\"
"
    );
}

fn resolve_watch_interval(
    args: &CliArgs,
    config: Option<&crate::config::Config>,
) -> Option<Duration> {
    match args.watch {
        Some(Some(interval)) => Some(interval),
        Some(None) => Some(
            config
                .map(|config| config.watch_interval)
                .unwrap_or_else(|| Duration::from_secs(5 * 60)),
        ),
        None => None,
    }
}

fn resolve_source_plan(
    args: CliArgs,
    config: Option<crate::config::Config>,
) -> io::Result<Vec<SourcePlan>> {
    if args.all && !args.sources.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--all cannot be combined with source flags",
        ));
    }

    if args.best && args.all {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--best cannot be combined with --all",
        ));
    }

    if args.best && !args.sources.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--best cannot be combined with source flags",
        ));
    }

    if args.best && args.output_mode == OutputMode::Usage {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--best cannot be combined with --usage",
        ));
    }

    if args.best {
        return Ok(crate::get_limits::best_source_plan());
    }

    if args.all {
        return Ok(crate::get_limits::source_list_plan(Source::ALL.to_vec()));
    }

    if !args.sources.is_empty() {
        return Ok(crate::get_limits::source_list_plan(args.sources));
    }

    let Some(config) = config else {
        return Ok(crate::get_limits::default_source_plan());
    };

    if config.default_sources.is_empty() {
        Ok(crate::get_limits::default_source_plan())
    } else {
        Ok(crate::get_limits::source_list_plan(config.default_sources))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn parse(raw_args: &[&str]) -> CliArgs {
        parse_args(raw_args.iter().map(|value| value.to_string())).expect("args should parse")
    }

    #[test]
    fn uses_required_defaults_without_config() {
        let args = parse(&[]);
        let selected = resolve_source_plan(args, None).expect("defaults should resolve");

        assert_eq!(selected, crate::get_limits::default_source_plan());
    }

    #[test]
    fn explicit_flags_override_config_defaults() {
        let args = parse(&["--codex-cli", "--claude-local"]);
        let config = Config {
            default_sources: Source::DEFAULTS.to_vec(),
            watch_interval: Duration::from_secs(60),
        };
        let selected =
            resolve_source_plan(args, Some(config)).expect("explicit source flags should win");

        assert_eq!(
            selected,
            vec![
                SourcePlan::Single(Source::CodexCli),
                SourcePlan::Single(Source::ClaudeLocal)
            ]
        );
    }

    #[test]
    fn supports_best_flag_and_short_alias() {
        assert!(parse(&["--best"]).best);
        assert!(parse(&["-b"]).best);
    }

    #[test]
    fn best_flag_selects_best_source_plan() {
        let args = parse(&["--best"]);
        let selected = resolve_source_plan(args, None).expect("best plan should resolve");

        assert_eq!(selected, crate::get_limits::best_source_plan());
    }

    #[test]
    fn supports_claude_statusline_flag() {
        let args = parse(&["--claude-statusline"]);

        assert_eq!(args.sources, vec![Source::ClaudeStatusline]);
    }

    #[test]
    fn limits_output_is_default() {
        let args = parse(&[]);

        assert_eq!(args.output_mode, OutputMode::Limits);
    }

    #[test]
    fn supports_usage_raw_and_structured_output_flags() {
        assert_eq!(parse(&["--usage"]).output_mode, OutputMode::Usage);
        assert_eq!(parse(&["--raw"]).output_mode, OutputMode::Raw);
        assert_eq!(parse(&["-r"]).output_mode, OutputMode::Raw);
        assert_eq!(parse(&["--structured"]).output_mode, OutputMode::Structured);
        assert_eq!(parse(&["-s"]).output_mode, OutputMode::Structured);
    }

    #[test]
    fn supports_watch_flag_with_optional_interval() {
        assert_eq!(parse(&["--watch"]).watch, Some(None));
        assert_eq!(parse(&["-w"]).watch, Some(None));
        assert_eq!(
            parse(&["--watch=10m"]).watch,
            Some(Some(Duration::from_secs(10 * 60)))
        );
        assert_eq!(
            parse(&["-w=30s"]).watch,
            Some(Some(Duration::from_secs(30)))
        );
    }

    #[test]
    fn supports_test_notification_flag() {
        assert_eq!(
            parse(&["--test-notification=75"]).test_notification,
            Some(LimitNotificationKind::Remaining75)
        );
        assert!(parse_args(["--test-notification=30"].into_iter().map(String::from)).is_err());
    }

    #[test]
    fn rejects_invalid_watch_interval_arg() {
        assert!(parse_args(["--watch=10"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["--watch=0s"].into_iter().map(String::from)).is_err());
    }

    #[test]
    fn resolves_watch_interval_from_flag_config_and_default() {
        let config = Config {
            default_sources: Vec::new(),
            watch_interval: Duration::from_secs(20),
        };

        assert_eq!(
            resolve_watch_interval(&parse(&["--watch=30s"]), Some(&config)),
            Some(Duration::from_secs(30))
        );
        assert_eq!(
            resolve_watch_interval(&parse(&["--watch"]), Some(&config)),
            Some(Duration::from_secs(20))
        );
        assert_eq!(
            resolve_watch_interval(&parse(&["--watch"]), None),
            Some(Duration::from_secs(5 * 60))
        );
        assert_eq!(resolve_watch_interval(&parse(&[]), Some(&config)), None);
    }

    #[test]
    fn rejects_combined_output_flags() {
        assert!(parse_args(["--raw", "--structured"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["--usage", "--raw"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["-s", "-r"].into_iter().map(String::from)).is_err());
    }

    #[test]
    fn rejects_best_with_all_usage_or_source_flags() {
        assert!(resolve_source_plan(parse(&["--best", "--all"]), None).is_err());
        assert!(resolve_source_plan(parse(&["--best", "--usage"]), None).is_err());
        assert!(resolve_source_plan(parse(&["--best", "--claude-local"]), None).is_err());
    }

    #[test]
    fn confirm_overwrite_accepts_y_and_yes() {
        assert!(read_overwrite_confirmation(
            Path::new("/tmp/config.toml"),
            &mut b"y\n".as_ref(),
            &mut Vec::new()
        )
        .expect("y should confirm"));
        assert!(read_overwrite_confirmation(
            Path::new("/tmp/config.toml"),
            &mut b"yes\n".as_ref(),
            &mut Vec::new()
        )
        .expect("yes should confirm"));
        assert!(read_overwrite_confirmation(
            Path::new("/tmp/config.toml"),
            &mut b"Y\n".as_ref(),
            &mut Vec::new()
        )
        .expect("Y should confirm"));
    }

    #[test]
    fn confirm_overwrite_rejects_empty_and_no() {
        assert!(!read_overwrite_confirmation(
            Path::new("/tmp/config.toml"),
            &mut b"\n".as_ref(),
            &mut Vec::new()
        )
        .expect("empty answer should decline"));
        assert!(!read_overwrite_confirmation(
            Path::new("/tmp/config.toml"),
            &mut b"n\n".as_ref(),
            &mut Vec::new()
        )
        .expect("n should decline"));
    }
}
