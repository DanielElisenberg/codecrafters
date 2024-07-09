use anyhow::Context;
use std::process;

use crate::environment;

fn run_child_process(command: &str, command_args: Vec<&str>) -> anyhow::Result<()> {
    let mut child_process = process::Command::new(command)
        .args(command_args)
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .spawn()?;
    // Wait for the child process to finish
    match child_process.wait() {
        Ok(exit_code) => match exit_code.code() {
            Some(_code) => Ok(()),
            None => Ok(()),
        },
        Err(e) => Err(e).context("Failed to wait for child process"),
    }
}

pub fn run_binary(command: &str, command_args: Vec<&str>) -> anyhow::Result<()> {
    for path in environment::get_path() {
        let binary_path = path.join(command);
        if binary_path.exists() {
            return run_child_process(binary_path.to_str().unwrap(), command_args.clone())
                .with_context(|| format!("Failed to run {}", command));
        };
    }
    anyhow::bail!("Could not find {}", command);
}
