# cargo-run-copy

[![Crates.io](https://img.shields.io/crates/v/cargo-run-copy.svg)](https://crates.io/crates/cargo-run-copy)
[![Actions Status](https://github.com/frozenlib/cargo-run-copy/workflows/CI/badge.svg)](https://github.com/frozenlib/cargo-run-copy/actions)

Provides functionality similar to `cargo run`, but instead of directly executing the binary, it copies it to a different location before running.

This tool addresses a Windows limitation where files cannot be updated while they are running. This restriction prevents running another `cargo run` while one is already in progress, which can reduce development efficiency.

## Use Case

For example, when developing a Model Context Protocol Server using the [`mcp-attr`](https://github.com/frozenlib/mcp-attr) crate, you can test the MCP server in development by registering it with the MCP client using `cargo run --manifest-path=<server's Cargo.toml> -- <server arguments>`.

However, if the server is registered with multiple clients, you need to either disable or unregister the server from all clients before rebuilding (some MCP clients may not stop the server just by disabling it). This can be quite cumbersome.

By using `cargo-run-copy` instead of `cargo run`, you can rebuild even while the server is running, significantly improving the development experience. With many MCP clients, you can reflect changes by simply pressing the MCP server update button after modifying the source code.

## How It Works

1. Runs `cargo build` to generate the executable
2. Calculates a hash value of the generated executable
3. Copies the executable to the `target/run-copy/<hash>/` directory
4. Runs the copied executable with the specified arguments

This approach allows new builds to proceed without locking the original executable.

## Installation

```sh
cargo install cargo-run-copy
```

## Usage

Use `cargo-run-copy` instead of `cargo run`:

```sh
cargo-run-copy [cargo build options] -- [program arguments]
```

When explicitly using a subcommand, use `run`:

```sh
cargo-run-copy run [cargo build options] -- [program arguments]
```

For compatibility, omitting the subcommand is equivalent to `run`.

## Command Reference

### `run`

```sh
cargo-run-copy run [cargo build options] -- [program arguments]
```

Runs `cargo build`, copies the generated executable to `target/run-copy/<hash>/`, and then runs the copied executable.

Calling `cargo-run-copy` without a subcommand is equivalent to this `run` command.

```sh
cargo-run-copy [cargo build options] -- [program arguments]
```

### `build`

```sh
cargo-run-copy build --exe-path-file <path> -- [cargo build options]
```

Runs `cargo build` and copies the generated executable to `target/run-copy/<hash>/`. It then writes the relative path of the copied executable to the file specified by `--exe-path-file`.

`--exe-path-file` is required. The written path is relative to the current directory where `cargo-run-copy` was executed.

If the build fails, the file specified by `--exe-path-file` is not updated.

### `run-from`

```sh
cargo-run-copy run-from --exe-path-file <path> -- [program arguments]
```

Reads the relative path of a copied executable from the file specified by `--exe-path-file`, then runs that executable with the specified arguments.

`--exe-path-file` is required. The relative path in the file is resolved from the current directory where `cargo-run-copy` was executed.

## Using with watchexec

When `watchexec --restart` runs the normal `cargo-run-copy run` command, the server is stopped first and the build starts afterward. If the build takes a long time, the server stays stopped during that time.

By splitting the workflow into `build` and `run-from`, the server can be restarted only after a successful build.

1. Run `cargo-run-copy build` from a `watchexec` process that watches source files
2. Update `--exe-path-file` only when `build` succeeds
3. Run `cargo-run-copy run-from` from a separate `watchexec --restart` process that watches `--exe-path-file`

Example:

```sh
watchexec -w src -w Cargo.toml -w Cargo.lock -i target -i .cargo-run-copy --on-busy-update=queue -- cargo-run-copy build --exe-path-file .cargo-run-copy/current-exe -- --manifest-path path/to/Cargo.toml
```

```sh
watchexec -w .cargo-run-copy/current-exe --restart -- cargo-run-copy run-from --exe-path-file .cargo-run-copy/current-exe -- [program arguments]
```

The file specified by `--exe-path-file` must exist before `run-from` can start. If needed, run `build` once before starting the watchers.

For the source watcher, exclude `target` and the directory containing `--exe-path-file`. Otherwise, copied executables or state file updates may trigger unnecessary rebuilds.

For the source watcher, `--on-busy-update=queue` is recommended so changes that happen during a build are queued for the next build.

Start both `watchexec` processes from the same current directory. The executable path written to `--exe-path-file` is treated as relative to the current directory.

## `cargo-run-copy-no-console`

When you run `cargo install cargo-run-copy`, the following two binaries will be installed:

- `cargo-run-copy`
- `cargo-run-copy-no-console`

On Windows, there are differences in behavior as shown below. (On platforms other than Windows, there is no difference.)

||`cargo-run-copy`|`cargo-run-copy-no-console`|
|---|---|---|
|Shows a console window|Yes|No|
|Returns control to the shell immediately when run from a shell|Yes|No|
|Displays standard output when run from a shell|Yes|No|

### Use case for `cargo-run-copy-no-console`

When using `cargo-run-copy` in the MCP server settings in Cursor, an unnecessary console window may appear. By using `cargo-run-copy-no-console` instead, you can suppress the display of the console window.

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
