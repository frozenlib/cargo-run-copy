#![windows_subsystem = "windows"]

fn main() -> anyhow::Result<()> {
    cargo_run_copy::run(true)
}
