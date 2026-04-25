# `build` / `run` / `run-from` サブコマンド追加計画

## 背景

`cargo-run-copy` は現在、`cargo build` を実行し、生成された実行ファイルを `target/run-copy/<hash>/` 以下へコピーしてから実行する。

この動作は Windows で実行中の exe が更新できない問題を避けるために有効だが、`watchexec` と組み合わせてサーバを再起動する用途では次の順序になりやすい。

1. サーバ停止
2. ビルド
3. サーバ起動

ビルド時間が長い場合、サーバ停止期間が長くなる。これを避けるため、ビルドとサーバ再起動を分離し、次の順序で運用できるようにする。

1. ビルド
2. サーバ停止
3. サーバ起動

## 目的

`watchexec` を 2 つ起動して、ソースコード監視と実行ファイルパス監視を分けられるようにする。

- ソースコード監視側は `cargo-run-copy build` を実行する
- `build` はビルド成功後にコピー済み exe の相対パスをファイルへ書く
- 実行ファイルパス監視側は、そのファイル更新を検知して `cargo-run-copy run-from` を実行する
- ビルド失敗時はファイルを更新しないため、既存サーバを停止しない

## CLI 仕様

### `build`

```sh
cargo-run-copy build --exe-path-file=<path> -- <cargo build args>
```

動作:

1. `cargo build --message-format=json-render-diagnostics` を実行する
2. 生成された実行ファイルを取得する
3. 実行ファイルの内容から hash を計算する
4. `target/run-copy/<hash>/<exe-name>` へコピーする
5. コピー済み exe の相対パスを `--exe-path-file` で指定されたファイルに書く

`--exe-path-file` は必須とする。

書き込むパスは、カレントディレクトリ基準の相対パスとする。

### `run`

```sh
cargo-run-copy run <cargo build args> -- <bin args>
```

動作:

現在の `cargo-run-copy <cargo build args> -- <bin args>` と同等。

1. build
2. copy
3. copied exe を実行

`run` は `--exe-path-file` を必要としない。

### `run-from`

```sh
cargo-run-copy run-from --exe-path-file=<path> -- <bin args>
```

動作:

1. `--exe-path-file` で指定されたファイルからコピー済み exe の相対パスを読む
2. その exe を `<bin args>` 付きで実行する
3. 子プロセスの終了コードを `cargo-run-copy` の終了コードとして返す

`--exe-path-file` は必須とする。

### サブコマンドなし

```sh
cargo-run-copy <cargo build args> -- <bin args>
```

互換性維持のため、サブコマンドなしは `run` と同等に扱う。

先頭引数が `build` / `run` / `run-from` のいずれかであればサブコマンドとして扱う。それ以外は従来通り `run` として扱う。

## `watchexec` 利用例

ソースコード監視:

```sh
watchexec -w src -w Cargo.toml -w Cargo.lock -- \
  cargo-run-copy build --exe-path-file=.cargo-run-copy/current-exe -- --manifest-path path/to/Cargo.toml
```

実行ファイルパス監視:

```sh
watchexec -w .cargo-run-copy/current-exe --restart -- \
  cargo-run-copy run-from --exe-path-file=.cargo-run-copy/current-exe -- <bin args>
```

## 実装方針

### 既存処理の分解

現在の `run` 処理を次の単位へ分ける。

- CLI 引数解析
- `cargo build` 実行
- build artifact から exe パス取得
- exe の hash 計算
- `target/run-copy/<hash>/<exe-name>` へのコピー
- exe 相対パスの書き込み
- exe 実行

既存の `run(connect_console: bool)` は、CLI 解析後に `run` サブコマンドの実装へ委譲する。

### `--exe-path-file` の扱い

`build` と `run-from` だけで受け付ける。

`build` では、コピー済み exe の相対パスを書き込む。監視側が途中書き込みを読まないよう、次の手順で atomic write する。

1. 同じディレクトリに一時ファイルを書く
2. flush する
3. 一時ファイルを `--exe-path-file` へ rename する

親ディレクトリが存在しない場合は作成する。

### 相対パスの基準

`build` が書き込む exe パスは、`cargo-run-copy` 実行時のカレントディレクトリ基準とする。

`run-from` も、ファイルから読んだ相対パスをカレントディレクトリ基準で解決する。

これにより、`watchexec` から同じ作業ディレクトリで起動すれば直感的に動く。

### 改行と空白

`--exe-path-file` には exe パス 1 行だけを書く。`run-from` は読み込んだ内容の前後空白と末尾改行を除去して扱う。

## エラー方針

- `build` で cargo build が失敗した場合、`--exe-path-file` は更新しない
- `build` で実行ファイルが生成されなかった場合はエラー
- `run-from` で `--exe-path-file` が存在しない場合はエラー
- `run-from` で読んだ exe が存在しない場合はエラー
- `run-from` の子プロセスが終了コードを返した場合、その終了コードで終了する
- 子プロセスがシグナル等で終了コードを返さない場合はエラー

## 互換性

既存の次の呼び出しは動作を変えない。

```sh
cargo-run-copy <cargo build args> -- <bin args>
```

ただし、先頭の cargo build 引数として `build` / `run` / `run-from` という裸の値を渡すケースはサブコマンドと解釈される。この衝突は実用上ほぼ問題ない想定。

## テスト観点

- サブコマンドなしが従来の `run` として解釈される
- `run` サブコマンドが従来の `run` と同じ引数分割を行う
- `build --exe-path-file=... -- <args>` が exe パスファイルを作成する
- `build` 失敗時に既存の exe パスファイルを更新しない
- `run-from --exe-path-file=... -- <args>` がファイル内の exe を実行する
- `run-from` が子プロセスの終了コードを伝播する
- `--exe-path-file` の親ディレクトリが存在しない場合に作成される
- exe パスファイルの末尾改行を許容する

## README 更新

実装後、`README.ja.md` と `README.md` に次を追記する。

- サブコマンド一覧
- 既存形式が `run` と同等であること
- `build` / `run-from` を使った `watchexec` 連携例
- `--exe-path-file` に書かれるパスの基準
