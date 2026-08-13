mod args;
mod help;
mod render;
mod run;

use std::io;
use std::process::ExitCode;

use crate::infra::loader::{TerminalStatus, TerminalUi};

use args::{parse_args, resolve_source_plan};
use help::print_help;
use run::run_sources_with_terminal_ui;

pub fn run() -> ExitCode {
    run_with_args(std::env::args().skip(1))
}

pub fn run_with_args(args: impl Iterator<Item = String>) -> ExitCode {
    match run_cli(args) {
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

fn run_cli(raw_args: impl Iterator<Item = String>) -> io::Result<TerminalStatus> {
    let args = parse_args(raw_args)?;

    if args.help {
        let mut ui = TerminalUi::new();
        ui.print_top()?;
        print_help();
        ui.print_bottom(TerminalStatus::Done)?;
        return Ok(TerminalStatus::Done);
    }

    let output_mode = args.output_mode;
    let plan = resolve_source_plan(args)?;

    let mut ui = TerminalUi::new();
    ui.print_top()?;
    let status = run_sources_with_terminal_ui(&mut ui, &plan, output_mode)?;
    ui.print_bottom(status)?;
    Ok(status)
}
