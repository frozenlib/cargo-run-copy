#![windows_subsystem = "windows"]

use std::{
    env,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio, exit},
};

use anyhow::bail;
use cargo_metadata::Message;

fn main() -> anyhow::Result<()> {
    let mut build_args = Vec::new();
    let mut run_args = Vec::new();
    let mut is_run_arg = false;
    for arg in env::args().skip(1) {
        if !is_run_arg && arg == "--" {
            is_run_arg = true;
            continue;
        }
        if is_run_arg {
            run_args.push(arg);
        } else {
            build_args.push(arg);
        }
    }
    let exe = build(&build_args)?;
    let target = to_target_dir(&exe)?;

    let hash = to_hash(&exe)?;
    let Some(file_name) = exe.file_name() else {
        bail!("Couldn't get file name");
    };
    let exe_copied = target.join("run-copy").join(hash).join(file_name);
    if !exe_copied.exists() {
        if let Some(parent) = exe_copied.parent() {
            std::fs::create_dir_all(parent)?;
            std::fs::copy(&exe, &exe_copied)?;
        }
    }
    eprintln!("     Running {}", exe_copied.display());
    let mut child = no_window(&mut Command::new(exe_copied))
        .args(run_args)
        .spawn()?;
    let output = child.wait()?;
    if let Some(code) = output.code() {
        exit(code);
    } else {
        bail!("Process terminated: {output}");
    }
}

fn build(build_args: &[String]) -> anyhow::Result<PathBuf> {
    let mut command = no_window(&mut Command::new("cargo"))
        .args(["build", "--message-format=json-render-diagnostics"])
        .args(build_args)
        .stdout(Stdio::piped())
        .spawn()?;

    let reader = BufReader::new(command.stdout.take().unwrap());
    let mut build_executable = None;

    for message in Message::parse_stream(reader) {
        match message? {
            Message::CompilerMessage(_) => {}
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    build_executable = Some(executable);
                }
            }
            Message::BuildScriptExecuted(_) => {}
            Message::BuildFinished(_) => {}
            _ => (),
        }
    }
    let output = command.wait()?;
    if !output.success() {
        if let Some(code) = output.code() {
            exit(code);
        } else {
            bail!("Cargo build failed");
        }
    }
    if let Some(executable) = build_executable {
        Ok(executable.into())
    } else {
        bail!("Cargo build failed to produce an executable");
    }
}

#[cfg(not(windows))]
fn no_window(command: &mut Command) -> &mut Command {
    command
}

#[cfg(windows)]
fn no_window(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    command.creation_flags(CREATE_NO_WINDOW)
}

fn to_target_dir(mut path: &Path) -> anyhow::Result<PathBuf> {
    loop {
        if path.is_dir() {
            if let Some(file_name) = path.file_name() {
                if file_name == "target" {
                    return Ok(path.to_path_buf());
                }
            }
        }
        if let Some(parent) = path.parent() {
            path = parent;
        } else {
            bail!("Couldn't find target directory");
        }
    }
}

fn to_hash(path: &Path) -> anyhow::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 8192];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}
