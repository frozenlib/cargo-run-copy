use std::{
    env,
    fs::{File, rename},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio, exit},
};

use anyhow::{Context, bail};
use cargo_metadata::Message;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, PartialEq, Eq)]
enum CliCommand {
    Build {
        exe_path_file: PathBuf,
        build_args: Vec<String>,
    },
    Run {
        build_args: Vec<String>,
        run_args: Vec<String>,
    },
    RunFrom {
        exe_path_file: PathBuf,
        run_args: Vec<String>,
    },
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: ClapCommand,
}

#[derive(Debug, Subcommand)]
enum ClapCommand {
    Build(BuildCommand),
    Run(RunCommand),
    RunFrom(RunFromCommand),
}

#[derive(Debug, Args)]
struct BuildCommand {
    #[arg(long)]
    exe_path_file: PathBuf,
    #[arg(last = true, num_args = 0.., allow_hyphen_values = true)]
    build_args: Vec<String>,
}

#[derive(Debug, Args)]
struct RunCommand {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Debug, Args)]
struct RunFromCommand {
    #[arg(long)]
    exe_path_file: PathBuf,
    #[arg(last = true, num_args = 0.., allow_hyphen_values = true)]
    run_args: Vec<String>,
}

pub fn run(connect_console: bool) -> anyhow::Result<()> {
    let command = match parse_args(env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => error.exit(),
    };

    match command {
        CliCommand::Build {
            exe_path_file,
            build_args,
        } => {
            let exe_copied = build_copy(&build_args, connect_console)?;
            write_exe_path_file(&exe_path_file, &to_current_dir_relative(&exe_copied)?)?;
        }
        CliCommand::Run {
            build_args,
            run_args,
        } => {
            let exe_copied = build_copy(&build_args, connect_console)?;
            run_executable(&exe_copied, &run_args, connect_console)?;
        }
        CliCommand::RunFrom {
            exe_path_file,
            run_args,
        } => {
            let exe = read_exe_path_file(&exe_path_file)?;
            run_executable(&exe, &run_args, connect_console)?;
        }
    }
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliCommand, clap::Error> {
    let args: Vec<_> = args.into_iter().collect();
    let subcommand = args.first().map(String::as_str);
    let args = if matches!(
        subcommand,
        Some("build" | "run" | "run-from" | "-h" | "--help" | "-V" | "--version")
    ) {
        args
    } else {
        std::iter::once("run".to_owned()).chain(args).collect()
    };

    match Cli::try_parse_from(std::iter::once("cargo-run-copy".to_owned()).chain(args))?.command {
        ClapCommand::Build(command) => Ok(CliCommand::Build {
            exe_path_file: command.exe_path_file,
            build_args: command.build_args,
        }),
        ClapCommand::Run(command) => {
            let (build_args, run_args) = split_args(command.args);
            Ok(CliCommand::Run {
                build_args,
                run_args,
            })
        }
        ClapCommand::RunFrom(command) => Ok(CliCommand::RunFrom {
            exe_path_file: command.exe_path_file,
            run_args: command.run_args,
        }),
    }
}

fn split_args(args: impl IntoIterator<Item = String>) -> (Vec<String>, Vec<String>) {
    let mut build_args = Vec::new();
    let mut run_args = Vec::new();
    let mut is_run_arg = false;

    for arg in args {
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

    (build_args, run_args)
}

fn build_copy(build_args: &[String], connect_console: bool) -> anyhow::Result<PathBuf> {
    let exe = build(build_args, connect_console)?;
    let target = to_target_dir(&exe)?;
    let hash = to_hash(&exe)?;
    let Some(file_name) = exe.file_name() else {
        bail!("Couldn't get file name");
    };
    let exe_copied = target.join("run-copy").join(hash).join(file_name);
    if !exe_copied.exists()
        && let Some(parent) = exe_copied.parent()
    {
        std::fs::create_dir_all(parent)?;
        std::fs::copy(&exe, &exe_copied)?;
    }

    Ok(exe_copied)
}

fn run_executable(exe: &Path, run_args: &[String], connect_console: bool) -> anyhow::Result<()> {
    eprintln!("     Running {}", exe.display());
    let mut child = apply_options(&mut Command::new(exe), connect_console)
        .args(run_args)
        .spawn()?;
    let output = child.wait()?;
    if let Some(code) = output.code() {
        exit(code);
    } else {
        bail!("Process terminated: {output}");
    }
}

fn write_exe_path_file(exe_path_file: &Path, exe: &Path) -> anyhow::Result<()> {
    if let Some(parent) = exe_path_file.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let file_name = exe_path_file
        .file_name()
        .context("--exe-path-file must not be a directory")?;
    let tmp_file = exe_path_file.with_file_name(format!(
        ".{}.{}.tmp",
        file_name.to_string_lossy(),
        std::process::id()
    ));

    {
        let mut file = File::create(&tmp_file)?;
        writeln!(file, "{}", exe.display())?;
        file.sync_all()?;
    }
    rename(&tmp_file, exe_path_file)?;

    Ok(())
}

fn read_exe_path_file(exe_path_file: &Path) -> anyhow::Result<PathBuf> {
    let content = std::fs::read_to_string(exe_path_file)?;
    let exe = content.trim();
    if exe.is_empty() {
        bail!("--exe-path-file is empty");
    }
    Ok(exe.into())
}

fn build(build_args: &[String], connect_console: bool) -> anyhow::Result<PathBuf> {
    let mut command = apply_options(&mut Command::new("cargo"), connect_console)
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

fn apply_options(command: &mut Command, connect_console: bool) -> &mut Command {
    if connect_console {
        command
    } else {
        no_window(command)
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
        if path.is_dir()
            && let Some(file_name) = path.file_name()
            && file_name == "target"
        {
            return Ok(path.to_path_buf());
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

fn to_current_dir_relative(path: &Path) -> anyhow::Result<PathBuf> {
    let path = path.canonicalize()?;
    let current_dir = env::current_dir()?.canonicalize()?;
    path_relative_from(&path, &current_dir).with_context(|| {
        format!(
            "Couldn't make {} relative to current directory",
            path.display()
        )
    })
}

fn path_relative_from(path: &Path, base: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let path_components: Vec<_> = path
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();
    let base_components: Vec<_> = base
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();

    let path_anchor: Vec<_> = path_components
        .iter()
        .take_while(|component| matches!(component, Component::Prefix(_) | Component::RootDir))
        .collect();
    let base_anchor: Vec<_> = base_components
        .iter()
        .take_while(|component| matches!(component, Component::Prefix(_) | Component::RootDir))
        .collect();
    if path_anchor != base_anchor {
        return None;
    }

    let mut common = 0;
    let max_common = path_components.len().min(base_components.len());
    while common < max_common && path_components[common] == base_components[common] {
        common += 1;
    }

    let mut result = PathBuf::new();
    for component in &base_components[common..] {
        if matches!(component, Component::Normal(_)) {
            result.push("..");
        }
    }
    for component in &path_components[common..] {
        result.push(component.as_os_str());
    }

    Some(result)
}

#[cfg(test)]
mod tests;
