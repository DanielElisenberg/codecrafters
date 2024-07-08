use adapters::{docker_hub, local_storage};
use anyhow::{Context, Result};
use libc;
use std::{env, process};
mod adapters;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let image_name = &args[2];
    let image_tag = "latest";
    let command = &args[3];
    let command_args = &args[4..];

    // Create sandbox directory to make sure the child process
    // doesn't have root access to our whole file system
    let sandbox_dir_path = local_storage::create_sandbox_dir()?;
    // the command argument contains the path to our executable,
    // so we need to copy this into our new sandbox
    local_storage::copy_command_binary(command, &sandbox_dir_path)?;
    // Get the compressed byte blobs from each layer in the manifest
    // decode to tar archives and extract to sanbox
    local_storage::extract_blobs(
        docker_hub::get_image_blobs_as_bytes(image_name, image_tag).await?,
        &sandbox_dir_path,
    )?;
    // Now we can chroot into the sandbox and make sure /dev/null
    // is set up.
    local_storage::setup_and_chroot(&sandbox_dir_path)?;

    // Unshare creates a new process namespace for new child processes
    // Marked unsafe as we are doing low-level system calls that change
    // process's namespace.
    unsafe {
        if libc::unshare(libc::CLONE_NEWPID) != 0 {
            return Err(anyhow::anyhow!("Failed to unshare PID namespace"));
        }
    }

    // Start the child process with arguments and use the same
    // stdout and stderr as parent
    let mut child_process = process::Command::new(command)
        .args(command_args)
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .spawn()?;
    // Wait for the child process to finish and exit with the
    // same status code
    match child_process.wait() {
        Ok(exit_code) => match exit_code.code() {
            Some(code) => process::exit(code),
            None => process::exit(1),
        },
        Err(e) => Err(e).context("Failed to wait for child process"),
    }
}
