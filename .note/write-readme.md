# Readme 作成指示

このツールの README.ja.md を作成してください

以下にこのツールに関して簡単に説明します

cargo run と同じように動作するが、実行ファイルを直接実行するではなく、別の場所にコピーしてから実行する

なぜそれが必要なのかというと、Windows では実行中のファイルを更新できない為、cargo run の実行中に cargo run を実行できないから。
そのために開発が不便になっていた

例えば、`mcp-attr`crate を使用した Model Context Protocol Server を開発する場合、MCP クライアントに `cargo run --manifest-path=<開発中のサーバのCargo.toml> -- <サーバの引数>` と登録することで、開発中の MCP サーバを実際に動作させてテストする場合を考える。

もし、複数のクライアントに登録していた場合、リビルドを行うには全てのクライアントで無効化、あるいは登録解除（MCP クライアントによっては無効化だけではサーバが停止しない場合がある）を行う必要があり、とても面倒。

`cargo run` の代わりに`cargo-run-copy`を使用することで、サーバが実行中でもリビルドできるようになり、開発体験が向上する。多くの MCP クライアントでは、ソースを変更した後、MCP サーバの更新ボタンを押すことで、ソースの変更を反映させることができるようになる。

## 動作の詳細

`main.rs`　を確認して、動作の readme に簡潔に書く。
