//! tchart CLI entry point.
//!
//! See `docs/spec/cli.md` for the full interface contract.
//!
//! Exit code mapping: `docs/spec/cli.md` §終了コード.
//! clap argument errors are caught via `try_parse()` and re-mapped to exit
//! code 1 (input error) instead of clap's default exit code 2.
//! `--help` and `--version` print to stdout with exit code 0 as usual.

use std::process::ExitCode;

use clap::Parser;

use tchart_cli::cli::{Cli, dispatch};

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => dispatch(cli),
        Err(error) => handle_parse_error(error),
    }
}

/// Handle a clap parse error.
///
/// `--help` and `--version` produce a `clap::Error` with `kind ==
/// DisplayHelp` / `DisplayVersion`; these are not real errors and exit with
/// code 0.
///
/// Running with no subcommand (`DisplayHelpOnMissingArgumentOrSubcommand`)
/// prints the help text but exits with code 1, as specified.
/// All other argument errors also map to exit code 1 (input error).
fn handle_parse_error(error: clap::Error) -> ExitCode {
    use clap::error::ErrorKind;
    let kind = error.kind();
    let is_success = kind == ErrorKind::DisplayHelp || kind == ErrorKind::DisplayVersion;
    let message = error.render();
    if is_success {
        print!("{message}");
        ExitCode::SUCCESS
    } else {
        eprint!("{message}");
        ExitCode::from(1u8)
    }
}
