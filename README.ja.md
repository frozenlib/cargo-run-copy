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

### `cargo-run-copy-no-console`

`cargo install cargo-run-copy` 実行すると次の二つのバイナリがインストールされます。

- `cargo-run-copy`
- `cargo-run-copy-no-console`

Windowsでは下記のような動作の違いがあります。（Windows以外では動作に違いはありません。）

||`cargo-run-copy`|`cargo-run-copy-no-console`|
|---|---|---|
|コンソールウィンドウを表示する|Yes|No|
|シェルから実行したときにすぐに制御を返す|Yes|No|
|シェルから実行したときに標準出力を表示する|Yes|No|

#### `cargo-run-copy-no-console` のユースケース

Cursorではmcpサーバの設定で `cargo-run-copy` を使用すると不必要なコンソールウィンドウが表示されてしまいますが、代わりに `cargo-run-copy-no-console` を使用することでコンソールウィンドウの表示を抑止することができます。

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
