use std::io;

use crate::get_limits::SourcePlan;
use crate::types::Source;

pub(super) struct CliArgs {
    pub help: bool,
    pub all: bool,
    pub best: bool,
    pub output_mode: OutputMode,
    pub sources: Vec<Source>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutputMode {
    Limits,
    Usage,
    Raw,
    Structured,
}

pub(super) fn parse_args(args: impl Iterator<Item = String>) -> io::Result<CliArgs> {
    let mut parsed = CliArgs {
        help: false,
        all: false,
        best: false,
        output_mode: OutputMode::Limits,
        sources: Vec::new(),
    };
    let mut output_mode = None;

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                parsed.help = true;
            }
            "-a" | "--all" => {
                parsed.all = true;
            }
            "-b" | "--best" => {
                parsed.best = true;
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
            "--codex-rpc" => {
                parsed.sources.push(Source::CodexRpc);
            }
            "--codex-cli" => {
                parsed.sources.push(Source::CodexCli);
            }
            "--claude-rpc" => {
                parsed.sources.push(Source::ClaudeRpc);
            }
            "--claude-cli" => {
                parsed.sources.push(Source::ClaudeCli);
            }
            "--claude-local" => {
                parsed.sources.push(Source::ClaudeLocal);
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

    if let Some(output_mode) = output_mode {
        parsed.output_mode = output_mode;
    }

    Ok(parsed)
}

pub(super) fn resolve_source_plan(args: CliArgs) -> io::Result<Vec<SourcePlan>> {
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

    Ok(crate::get_limits::default_source_plan())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw_args: &[&str]) -> CliArgs {
        parse_args(raw_args.iter().map(|value| value.to_string())).expect("args should parse")
    }

    #[test]
    fn uses_built_in_defaults_without_source_flags() {
        let args = parse(&[]);
        let selected = resolve_source_plan(args).expect("defaults should resolve");

        assert_eq!(selected, crate::get_limits::default_source_plan());
    }

    #[test]
    fn explicit_source_flags_select_those_sources() {
        let args = parse(&["--codex-rpc", "--claude-rpc", "--claude-local"]);
        let selected = resolve_source_plan(args).expect("explicit source flags should resolve");

        assert_eq!(
            selected,
            vec![
                SourcePlan::Single(Source::CodexRpc),
                SourcePlan::Single(Source::ClaudeRpc),
                SourcePlan::Single(Source::ClaudeLocal)
            ]
        );
    }

    #[test]
    fn legacy_cli_flags_stay_available_as_diagnostic_selectors() {
        let codex =
            resolve_source_plan(parse(&["--codex-cli"])).expect("legacy flag should resolve");
        let claude =
            resolve_source_plan(parse(&["--claude-cli"])).expect("legacy flag should resolve");

        assert_eq!(codex, vec![SourcePlan::Single(Source::CodexCli)]);
        assert_eq!(claude, vec![SourcePlan::Single(Source::ClaudeCli)]);
        assert!(!Source::ALL.contains(&Source::CodexCli));
        assert!(!Source::ALL.contains(&Source::ClaudeCli));
    }

    #[test]
    fn supports_best_flag_and_short_alias() {
        assert!(parse(&["--best"]).best);
        assert!(parse(&["-b"]).best);
    }

    #[test]
    fn best_flag_selects_best_source_plan() {
        let args = parse(&["--best"]);
        let selected = resolve_source_plan(args).expect("best plan should resolve");

        assert_eq!(selected, crate::get_limits::best_source_plan());
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
    fn rejects_watch_and_init_config_flags() {
        assert!(parse_args(["--watch"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["--watch=10m"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["-w"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["--init-config"].into_iter().map(String::from)).is_err());
    }

    #[test]
    fn rejects_combined_output_flags() {
        assert!(parse_args(["--raw", "--structured"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["--usage", "--raw"].into_iter().map(String::from)).is_err());
        assert!(parse_args(["-s", "-r"].into_iter().map(String::from)).is_err());
    }

    #[test]
    fn rejects_best_with_all_usage_or_source_flags() {
        assert!(resolve_source_plan(parse(&["--best", "--all"])).is_err());
        assert!(resolve_source_plan(parse(&["--best", "--usage"])).is_err());
        assert!(resolve_source_plan(parse(&["--best", "--claude-local"])).is_err());
    }
}
