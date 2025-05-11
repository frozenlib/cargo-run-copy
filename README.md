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
