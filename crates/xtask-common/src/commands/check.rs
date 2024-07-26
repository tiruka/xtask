use std::process::Command;

use anyhow::{anyhow, Ok, Result};
use clap::{Args, Subcommand};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

use crate::{
    endgroup, group,
    utils::{
        cargo::ensure_cargo_crate_is_installed,
        prompt::ask_once,
        workspace::{get_workspace_members, WorkspaceMemberType},
    }, versions::TYPOS_VERSION,
};

use super::Target;

#[derive(Args, Clone)]
pub struct CheckCmdArgs {
    /// Target to check for.
    #[arg(short, long, value_enum, default_value_t = Target::All)]
    target: Target,
    /// Comma-separated list of excluded crates.
    #[arg(
        short = 'x',
        long,
        value_name = "CRATE,CRATE,...",
        value_delimiter = ',',
        required = false
    )]
    pub exclude: Vec<String>,
    /// Comma-separated list of crates to include exclusively.
    #[arg(
        short = 'n',
        long,
        value_name = "CRATE,CRATE,...",
        value_delimiter = ',',
        required = false
    )]
    pub only: Vec<String>,
    #[command(subcommand)]
    pub command: CheckCommand,
}

#[derive(EnumString, EnumIter, Display, Clone, PartialEq, Subcommand)]
#[strum(serialize_all = "lowercase")]
pub enum CheckCommand {
    /// Run audit command.
    Audit,
    /// Run format command and fix formatting.
    Format,
    /// Run lint command and fix issues.
    Lint,
    /// Find typos in source code and fix them.
    Typos,
    /// Run all the checks.
    All,
}

pub fn handle_command(args: CheckCmdArgs, answer: Option<bool>) -> anyhow::Result<()> {
    match args.command {
        CheckCommand::Audit => run_audit(&args.target, answer),
        CheckCommand::Format => run_format(&args.target, &args.exclude, &args.only, answer),
        CheckCommand::Lint => run_lint(&args.target, &args.exclude, &args.only, answer),
        CheckCommand::Typos => run_typos(&args.target, answer),
        CheckCommand::All => {
            let answer = ask_once(
                "This will run all the checks with autofix on all members of the workspace.",
            );
            CheckCommand::iter()
                .filter(|c| *c != CheckCommand::All)
                .try_for_each(|c| {
                    handle_command(
                        CheckCmdArgs {
                            command: c,
                            target: args.target.clone(),
                            exclude: args.exclude.clone(),
                            only: args.only.clone(),
                        },
                        Some(answer),
                    )
                })
        }
    }
}

pub(crate) fn run_audit(target: &Target, mut answer: Option<bool>) -> anyhow::Result<()> {
    match target {
        Target::Crates | Target::Examples => {
            if answer.is_none() {
                answer = Some(ask_once(
                    "This will run the audit check with autofix mode enabled.",
                ));
            };
            if answer.unwrap() {
                ensure_cargo_crate_is_installed("cargo-audit", Some("fix"), None, false)?;
                group!("Audit: Crates and Examples");
                info!("Command line: cargo audit -q --color always fix");
                let status = Command::new("cargo")
                    .args(["audit", "-q", "--color", "always", "fix"])
                    .status()
                    .map_err(|e| anyhow!("Failed to execute cargo audit: {}", e))?;
                if !status.success() {
                    return Err(anyhow!("Audit check execution failed"));
                }
                endgroup!();
            }
        }
        Target::All => {
            if answer.is_none() {
                answer = Some(ask_once("This will run audit checks on all targets."));
            };
            Target::iter()
                .filter(|p| *p != Target::All && *p != Target::Examples)
                .try_for_each(|p| run_audit(&p, answer))?;
        }
    }
    Ok(())
}

fn run_format(
    target: &Target,
    excluded: &Vec<String>,
    only: &Vec<String>,
    mut answer: Option<bool>,
) -> Result<()> {
    match target {
        Target::Crates | Target::Examples => {
            let members = match target {
                Target::Crates => get_workspace_members(WorkspaceMemberType::Crate),
                Target::Examples => get_workspace_members(WorkspaceMemberType::Example),
                _ => unreachable!(),
            };

            if answer.is_none() {
                answer = Some(ask_once(&format!(
                    "This will run format checks on all {} of the workspace.",
                    if *target == Target::Crates {
                        "crates"
                    } else {
                        "examples"
                    }
                )));
            }

            if answer.unwrap() {
                for member in members {
                    group!("Format: {}", member.name);
                    if excluded.contains(&member.name)
                        || (!only.is_empty() && !only.contains(&member.name))
                    {
                        info!("Skip '{}' because it has been excluded!", &member.name);
                        continue;
                    }
                    info!("Command line: cargo fmt -p {} -- --color=always", &member.name);
                    let status = Command::new("cargo")
                        .args(["fmt", "-p", &member.name, "--", "--color=always"])
                        .status()
                        .map_err(|e| anyhow!("Failed to execute cargo fmt: {}", e))?;
                    if !status.success() {
                        return Err(anyhow!(
                            "Format check execution failed for {}",
                            &member.name
                        ));
                    }
                    endgroup!();
                }
            }
        }
        Target::All => {
            if answer.is_none() {
                answer = Some(ask_once(
                    "This will run format check on all members of the workspace.",
                ));
            }
            if answer.unwrap() {
                Target::iter()
                    .filter(|t| *t != Target::All)
                    .try_for_each(|t| run_format(&t, excluded, only, answer))?;
            }
        }
    }
    Ok(())
}

fn run_lint(
    target: &Target,
    excluded: &Vec<String>,
    only: &Vec<String>,
    mut answer: Option<bool>,
) -> anyhow::Result<()> {
    match target {
        Target::Crates | Target::Examples => {
            let members = match target {
                Target::Crates => get_workspace_members(WorkspaceMemberType::Crate),
                Target::Examples => get_workspace_members(WorkspaceMemberType::Example),
                _ => unreachable!(),
            };

            if answer.is_none() {
                answer = Some(ask_once(&format!(
                    "This will run lint fix on all {} of the workspace.",
                    if *target == Target::Crates {
                        "crates"
                    } else {
                        "examples"
                    }
                )));
            }

            if answer.unwrap() {
                for member in members {
                    group!("Lint: {}", member.name);
                    if excluded.contains(&member.name)
                        || (!only.is_empty() && !only.contains(&member.name))
                    {
                        info!("Skip '{}' because it has been excluded!", &member.name);
                        continue;
                    }
                    info!(
                        "Command line: cargo clippy --no-deps --fix --allow-dirty --allow-staged --color=always -p {} -- --deny warnings",
                        &member.name
                    );
                    let status = Command::new("cargo")
                        .args([
                            "clippy",
                            "--no-deps",
                            "--fix",
                            "--allow-dirty",
                            "--allow-staged",
                            "--color=always",
                            "-p",
                            &member.name,
                            "--",
                            "--deny",
                            "warnings"
                        ])
                        .status()
                        .map_err(|e| anyhow!("Failed to execute cargo clippy: {}", e))?;
                    if !status.success() {
                        return Err(anyhow!("Lint fix execution failed for {}", &member.name));
                    }
                    endgroup!();
                }
            }
        }
        Target::All => {
            if answer.is_none() {
                answer = Some(ask_once(
                    "This will run lint fix on all members of the workspace.",
                ));
            }
            if answer.unwrap() {
                Target::iter()
                    .filter(|t| *t != Target::All)
                    .try_for_each(|t| run_lint(&t, excluded, only, answer))?;
            }
        }
    }
    Ok(())
}

pub(crate) fn run_typos(target: &Target, mut answer: Option<bool>) -> anyhow::Result<()> {
    match target {
        Target::Crates | Target::Examples => {
            if answer.is_none() {
                answer = Some(ask_once(
                    "This will look for typos in the source code check and auto-fix them.",
                ));
            };
            if answer.unwrap() {
                ensure_cargo_crate_is_installed("typos-cli", None, Some(TYPOS_VERSION), false)?;
                group!("Typos: Crates and Examples");
                info!("Command line: typos --write-changes");
                let status = Command::new("typos")
                    .args(["--write-changes"])
                    .status()
                    .map_err(|e| anyhow!("Failed to execute typos: {}", e))?;
                if !status.success() {
                    return Err(anyhow!("Some typos have been found and cannot be fixed."));
                }
                endgroup!();
            }
        }
        Target::All => {
            if answer.is_none() {
                answer = Some(ask_once("This will look for typos on all targets."));
            };
            Target::iter()
                .filter(|p| *p != Target::All && *p != Target::Examples)
                .try_for_each(|p| run_typos(&p, answer))?;
        }
    }
    Ok(())
}
