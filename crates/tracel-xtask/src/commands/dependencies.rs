use anyhow::Ok;
use strum::IntoEnumIterator;

use crate::{
    commands::CARGO_NIGHTLY_MSG,
    endgroup, group,
    utils::{
        cargo::ensure_cargo_crate_is_installed, process::run_process,
        rustup::is_current_toolchain_nightly,
    },
};

#[tracel_xtask_macros::declare_command_args(None, DependenciesSubCommand)]
pub struct DependenciesCmdArgs {}

pub fn handle_command(args: DependenciesCmdArgs) -> anyhow::Result<()> {
    match args.get_command() {
        DependenciesSubCommand::Deny => run_cargo_deny(),
        DependenciesSubCommand::Unused => run_cargo_udeps(),
        DependenciesSubCommand::All => DependenciesSubCommand::iter()
            .filter(|c| *c != DependenciesSubCommand::All)
            .try_for_each(|c| handle_command(DependenciesCmdArgs { command: Some(c) })),
    }
}

/// Run cargo-deny
fn run_cargo_deny() -> anyhow::Result<()> {
    ensure_cargo_crate_is_installed("cargo-deny", None, None, false)?;
    // Run cargo deny
    group!("Cargo: run deny checks");
    run_process(
        "cargo",
        &vec!["deny", "check"],
        None,
        None,
        "Some dependencies don't meet the requirements!",
    )?;
    endgroup!();
    Ok(())
}

/// Run cargo-udeps
fn run_cargo_udeps() -> anyhow::Result<()> {
    if is_current_toolchain_nightly() {
        ensure_cargo_crate_is_installed("cargo-udeps", None, None, false)?;
        // Run cargo udeps
        group!("Cargo: run unused dependencies checks");
        run_process(
            "cargo",
            &vec!["udeps"],
            None,
            None,
            "Unused dependencies found!",
        )?;
        endgroup!();
    } else {
        error!("{}", CARGO_NIGHTLY_MSG);
    }
    Ok(())
}
