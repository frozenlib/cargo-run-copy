# cargo-run-copy

[![Crates.io](https://img.shields.io/crates/v/cargo-run-copy.svg)](https://crates.io/crates/cargo-run-copy)
[![Actions Status](https://github.com/frozenlib/cargo-run-copy/workflows/CI/badge.svg)](https://github.com/frozenlib/cargo-run-copy/actions)

`cargo run` と同様の機能を提供しますが、実行ファイルを直接実行するのではなく、別の場所にコピーしてから実行します。

これは、Windows では実行中のファイルを更新できないという制限に対処するためのツールです。この制限により、`cargo run` の実行中に別の `cargo run` を実行することができず、開発の効率が低下していました。

## ユースケース

例えば、 [`mcp-attr`](https://github.com/frozenlib/mcp-attr) クレートを使用した Model Context Protocol Server を開発する場合を考えてみましょう。MCP クライアントに `cargo run --manifest-path=<開発中のサーバのCargo.toml> -- <サーバの引数>` と登録することで、開発中の MCP サーバを実際に動作させてテストすることができます。

しかし、複数のクライアントに登録している場合、リビルドを行うには全てのクライアントでサーバを無効化するか登録解除する必要があります（MCP クライアントによっては無効化だけではサーバが停止しない場合があります）。これは非常に面倒な作業です。

`cargo run` の代わりに `cargo-run-copy` を使用することで、サーバが実行中でもリビルドが可能になり、開発体験が大幅に向上します。多くの MCP クライアントでは、ソースを変更した後、MCP サーバの更新ボタンを押すだけで変更を反映させることができます。

## 動作の仕組み

1. `cargo build` を実行して実行ファイルを生成します
2. 生成された実行ファイルのハッシュ値を計算します
3. `target/run-copy/<ハッシュ値>/` ディレクトリに実行ファイルをコピーします
4. コピーした実行ファイルを指定された引数で実行します

この方法により、元の実行ファイルをロックすることなく、新しいビルドを行うことができます。

## インストール

```sh
cargo install cargo-run-copy
```

## 使用方法

`cargo run` の代わりに `cargo-run-copy` を使用します：

```sh
cargo-run-copy [cargo buildのオプション] -- [プログラムの引数]
```

サブコマンドを明示する場合は、次のように `run` を使用します：

```sh
cargo-run-copy run [cargo buildのオプション] -- [プログラムの引数]
```

サブコマンドを省略した場合は、互換性のため `run` と同じ動作になります。

## コマンドリファレンス

### `run`

```sh
cargo-run-copy run [cargo buildのオプション] -- [プログラムの引数]
```

`cargo build` を実行し、生成された実行ファイルを `target/run-copy/<ハッシュ値>/` にコピーしてから実行します。

サブコマンドなしの呼び出しは、この `run` と同等です。

```sh
cargo-run-copy [cargo buildのオプション] -- [プログラムの引数]
```

### `build`

```sh
cargo-run-copy build --exe-path-file <path> -- [cargo buildのオプション]
```

`cargo build` を実行し、生成された実行ファイルを `target/run-copy/<ハッシュ値>/` にコピーします。その後、コピーした実行ファイルの相対パスを `--exe-path-file` で指定されたファイルに書き込みます。

`--exe-path-file` は必須です。書き込まれるパスは、`cargo-run-copy` を実行したカレントディレクトリ基準の相対パスです。

ビルドに失敗した場合、`--exe-path-file` で指定されたファイルは更新されません。

### `run-from`

```sh
cargo-run-copy run-from --exe-path-file <path> -- [プログラムの引数]
```

`--exe-path-file` で指定されたファイルからコピー済み実行ファイルの相対パスを読み取り、その実行ファイルを指定された引数で実行します。

`--exe-path-file` は必須です。ファイル内の相対パスは、`cargo-run-copy` を実行したカレントディレクトリ基準で解決されます。

## watchexec と併用する

`watchexec --restart` で通常の `cargo-run-copy run` を実行すると、変更検知時に先にサーバが停止し、その後にビルドが始まります。ビルド時間が長い場合、その間サーバが停止したままになります。

`build` と `run-from` を分けることで、ビルドが成功してからサーバを再起動できます。

1. ソースコード監視用の `watchexec` で `cargo-run-copy build` を実行する
2. `build` が成功した場合だけ `--exe-path-file` が更新される
3. `--exe-path-file` 監視用の `watchexec --restart` で `cargo-run-copy run-from` を実行する

例：

```sh
watchexec -w src -w Cargo.toml -w Cargo.lock -i target -i .cargo-run-copy --on-busy-update=queue -- cargo-run-copy build --exe-path-file .cargo-run-copy/current-exe -- [cargo buildのオプション]
```

```sh
watchexec -w .cargo-run-copy/current-exe --restart -- cargo-run-copy run-from --exe-path-file .cargo-run-copy/current-exe -- [プログラムの引数]
```

最初に `run-from` するためには、`--exe-path-file` で指定されたファイルが存在している必要があります。必要に応じて、監視を開始する前に一度 `build` を実行してください。

ソースコード監視側では、`target` や `--exe-path-file` を置くディレクトリを監視対象から除外してください。コピー先や状態ファイルの更新をソース変更として扱うと、不要な再ビルドが発生します。

`.cargo-run-copy` は生成される状態ファイルを置くためのディレクトリなので、`.gitignore` に追加することを推奨します。

ソースコード監視側では、ビルド中に追加の変更が発生した場合に次のビルドをキューへ積むため、`--on-busy-update=queue` を指定することを推奨します。

2 つの `watchexec` は同じカレントディレクトリで起動してください。`--exe-path-file` に書き込まれる実行ファイルパスは、カレントディレクトリ基準の相対パスとして扱われます。

## `cargo-run-copy-no-console`

`cargo install cargo-run-copy` 実行すると次の二つのバイナリがインストールされます。

- `cargo-run-copy`
- `cargo-run-copy-no-console`

Windowsでは下記のような動作の違いがあります。（Windows以外では動作に違いはありません。）

||`cargo-run-copy`|`cargo-run-copy-no-console`|
|---|---|---|
|コンソールウィンドウを表示する|Yes|No|
|シェルから実行したときにすぐに制御を返す|Yes|No|
|シェルから実行したときに標準出力を表示する|Yes|No|

### `cargo-run-copy-no-console` のユースケース

Cursorではmcpサーバの設定で `cargo-run-copy` を使用すると不必要なコンソールウィンドウが表示されてしまいますが、代わりに `cargo-run-copy-no-console` を使用することでコンソールウィンドウの表示を抑止することができます。

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
