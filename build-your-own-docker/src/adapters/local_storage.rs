use anyhow::Context;
use flate2::read::GzDecoder;
use std::{env, fs, io::Cursor, os, path::PathBuf};
use tar::Archive;

pub fn create_sandbox_dir() -> anyhow::Result<PathBuf> {
    let sandbox_dir = env::current_dir()?.join("sandbox");
    fs::create_dir_all(&sandbox_dir)?;
    Ok(sandbox_dir)
}

pub fn copy_command_binary(command: &str, sandbox_dir_path: &PathBuf) -> anyhow::Result<()> {
    let new_binary_dir = sandbox_dir_path.join(command.strip_prefix("/").unwrap());
    fs::create_dir_all(sandbox_dir_path.join("usr/local/bin"))?;
    fs::create_dir_all(sandbox_dir_path.join("bin"))?;
    fs::copy(command, new_binary_dir).with_context(|| "Failed to copy binary to sandbox")?;
    Ok(())
}

pub fn setup_and_chroot(sandbox_dir_path: &PathBuf) -> anyhow::Result<()> {
    os::unix::fs::chroot(sandbox_dir_path).with_context(|| "Failed to chroot into sandbox")?;
    env::set_current_dir("/")?;
    fs::create_dir_all("/dev")?;
    fs::File::create("/dev/null").with_context(|| "Failed to create /dev/null")?;
    Ok(())
}

pub fn extract_blobs(blobs: Vec<bytes::Bytes>, sandbox_dir_path: &PathBuf) -> anyhow::Result<()> {
    for blob in blobs {
        let cursor = Cursor::new(blob);
        let mut tar = Archive::new(GzDecoder::new(cursor));
        tar.unpack(&sandbox_dir_path)
            .with_context(|| "Failed to extract blob")?;
    }
    Ok(())
}
