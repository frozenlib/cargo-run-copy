use super::*;

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}

#[test]
fn parse_legacy_run_command() {
    assert_eq!(
        parse_args(strings(&["--release", "--", "server-arg"])).unwrap(),
        CliCommand::Run {
            build_args: strings(&["--release"]),
            run_args: strings(&["server-arg"])
        }
    );
}

#[test]
fn parse_explicit_run_command() {
    assert_eq!(
        parse_args(strings(&["run", "--bin", "server", "--", "server-arg"])).unwrap(),
        CliCommand::Run {
            build_args: strings(&["--bin", "server"]),
            run_args: strings(&["server-arg"])
        }
    );
}

#[test]
fn parse_build_command() {
    assert_eq!(
        parse_args(strings(&[
            "build",
            "--exe-path-file",
            ".cargo-run-copy/current-exe",
            "--",
            "--release"
        ]))
        .unwrap(),
        CliCommand::Build {
            exe_path_file: ".cargo-run-copy/current-exe".into(),
            build_args: strings(&["--release"])
        }
    );
}

#[test]
fn parse_build_command_with_equals_option() {
    assert_eq!(
        parse_args(strings(&[
            "build",
            "--exe-path-file=.cargo-run-copy/current-exe",
            "--",
            "--release"
        ]))
        .unwrap(),
        CliCommand::Build {
            exe_path_file: ".cargo-run-copy/current-exe".into(),
            build_args: strings(&["--release"])
        }
    );
}

#[test]
fn parse_run_from_command() {
    assert_eq!(
        parse_args(strings(&[
            "run-from",
            "--exe-path-file",
            ".cargo-run-copy/current-exe",
            "--",
            "server-arg"
        ]))
        .unwrap(),
        CliCommand::RunFrom {
            exe_path_file: ".cargo-run-copy/current-exe".into(),
            run_args: strings(&["server-arg"])
        }
    );
}

#[test]
fn exe_path_file_is_required_for_build() {
    assert!(parse_args(strings(&["build", "--", "--release"])).is_err());
}

#[test]
fn exe_path_file_is_required_for_run_from() {
    assert!(parse_args(strings(&["run-from", "--", "server-arg"])).is_err());
}

#[test]
fn path_relative_from_child_path() {
    let mut expected = PathBuf::new();
    expected.push("..");
    expected.push("b");
    expected.push("c");

    assert_eq!(
        path_relative_from(Path::new("a/b/c"), Path::new("a/d")).unwrap(),
        expected
    );
}

#[test]
fn path_relative_from_same_path() {
    assert_eq!(
        path_relative_from(Path::new("a/b"), Path::new("a/b")).unwrap(),
        PathBuf::new()
    );
}

#[test]
fn write_and_read_exe_path_file() {
    let dir = env::temp_dir().join(format!("cargo_run_copy_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let exe_path_file = dir.join("state").join("current-exe");
    let exe = Path::new("target/run-copy/hash/server");

    write_exe_path_file(&exe_path_file, exe).unwrap();

    assert_eq!(read_exe_path_file(&exe_path_file).unwrap(), exe);

    std::fs::remove_dir_all(dir).unwrap();
}
