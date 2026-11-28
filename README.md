# resource-locator

設定ファイルや環境変数に書いた「リソースの場所」を表す文字列を解析し、型付きの値に変換するRustクレートです。

- ファイルの場所: Cloud Storageのオブジェクト、またはローカルファイル
- シークレットの場所: Secret Managerのシークレット、環境変数、ローカルファイル、またはシークレットなし

このクレートは文字列の解析と検証だけを行います。ファイルやシークレットの中身を実際に読み込む処理は含みません。

## 使い方

`Cargo.toml`に追加します。

```toml
[dependencies]
resource-locator = { git = "https://github.com/kuroyasu67/resource-locator" }
```

文字列を`parse`すると、場所の種類に応じた値(`FileLocation`または`SecretLocation`)が得られます。

コード例はAPIドキュメントにあります。`cargo doc --open`で確認できます。

## 受け付ける形式

### ファイル(`FileLocation`)

| 形式 | 意味 |
| --- | --- |
| `gs://<bucket>/<object-path>` | Cloud Storageのオブジェクト |
| `local://<path>` | ローカルファイル(`local:///etc/app.toml`は絶対パス、`local://./app.toml`は相対パス) |

Cloud Storageのバケット名は全プロジェクトで一意なので、プロジェクトは指定しません。

### シークレット(`SecretLocation`)

| 形式 | 意味 |
| --- | --- |
| `none://` | シークレットなし |
| `env://<name>` | 環境変数 |
| `local://<path>` | ローカルファイル(パスの書き方は`FileLocation`と同じ) |
| `sm://<secret>` | Secret Manager(プロジェクトなし、バージョンは`latest`) |
| `sm://<secret>/<version>` | Secret Manager(プロジェクトなし) |
| `sm://<project>/<secret>/<version>` | Secret Manager(プロジェクトあり) |
| `sm://projects/<project>/secrets/<secret>` | Secret Managerのリソース名(バージョンは`latest`) |
| `sm://projects/<project>/secrets/<secret>/versions/<version>` | Secret Managerのリソース名 |

- `sm://`の形式は、`/`で区切った部分の数で判別します。部分が2つの場合はバージョンの指定とみなすため、プロジェクトを指定してバージョンを省略するときは`sm://<project>/<secret>/latest`と書きます。
- プロジェクトを省略した場合、どのプロジェクトのシークレットを読むかは利用側が決めます。
- `<project>`には、プロジェクトIDとプロジェクト番号のどちらも指定できます。

### 共通の規則

- 入力の前後と、`/`で区切った各部分の前後の空白は取り除きます。
- スキームは大文字と小文字を区別します(`GS://`は受け付けません)。

## 検証

解析時に、各部分がGoogle CloudやPOSIXの命名規則に従っているかを検証します。

| 対象 | 規則 |
| --- | --- |
| バケット名 | 英小文字・数字・`-`・`_`・`.`、先頭と末尾は英小文字か数字、3〜63文字(`.`を含む場合は全体で222文字以内) |
| プロジェクトID | 英小文字・数字・`-`、先頭は英小文字、末尾は`-`以外、6〜30文字(数字のみの場合はプロジェクト番号とみなす) |
| シークレット名 | 英数字・`-`・`_`、255文字以内 |
| バージョン | 英数字・`-`・`_`(番号、`latest`、エイリアス) |
| 環境変数名 | `[A-Za-z_][A-Za-z0-9_]*` |

## エラー

解析に失敗すると`resource_locator::Error`を返します。原因ごとにバリアントが分かれており、どのバリアントも入力全体を`location`に保持します。
バリアントは次のとおりです。

| バリアント | 原因 |
| --- | --- |
| `Empty` | 空、または空白のみ |
| `MissingScheme` | `<scheme>://`の形になっていない |
| `UnsupportedScheme` | 対応していないスキーム |
| `UnexpectedContent` | `none://`の後ろに文字列がある |
| `MalformedSecretPath` | `sm://`の後ろがどの形式にも一致しない |
| `MissingProject` / `InvalidProject` | プロジェクトが空 / 規則に違反 |
| `MissingSecretName` / `InvalidSecretName` | シークレット名が空 / 規則に違反 |
| `InvalidSecretVersion` | バージョンが規則に違反 |
| `MissingEnvVarName` / `InvalidEnvVarName` | 環境変数名が空 / 規則に違反 |
| `MissingBucket` / `InvalidBucketName` | バケット名が空 / 規則に違反 |
| `MissingObjectPath` | オブジェクトパスが空 |
| `MissingLocalPath` | ローカルパスが空(`FileLocation`、`SecretLocation`共通) |

`Error`には`#[non_exhaustive]`が付いているため、`match`するときはワイルドカード(`_`)の腕が必要です。

## 文字列への変換

どの型も`Display`を実装しており、正規化した形式で出力します。
出力した文字列を再び解析すると、元と同じ値になります。
たとえば`sm://projects/my-project/secrets/db-password`は`sm://my-project/db-password/latest`と出力します。

## serdeによるデシリアライズ

`serde`フィーチャーを有効にすると、`FileLocation`と`SecretLocation`が`serde::Deserialize`を実装します。
文字列を`parse`と同じ規則で解析するため、設定ファイルの構造体のフィールドにそのまま使えます。

```toml
[dependencies]
resource-locator = { git = "https://github.com/kuroyasu67/resource-locator", features = ["serde"] }
```

```rust
use resource_locator::{FileLocation, SecretLocation};

#[derive(serde::Deserialize)]
struct Config {
    template: FileLocation,
    api_key: SecretLocation,
}
```

解析に失敗した場合は、`Error`のメッセージがデシリアライザーのエラーとして返ります。

## 動作環境

Rust 1.85以降（`Cargo.toml`の`rust-version`）

## 開発

ブランチとリリース、Rustの版、リンター、Gitのフック、テスト、CI、依存関係の自動更新およびクレートの版の指定の方針は、[開発ツール](docs/development-tools.md)に記述しています。

## 依存関係の更新

依存関係の更新には[Dependabot](https://docs.github.com/ja/code-security/dependabot)を使っています。
設定は`.github/dependabot.yml`にあります。

### 更新の対象とタイミング

| 対象 | 内容 |
| --- | --- |
| `cargo` | `Cargo.toml`と`Cargo.lock`の依存クレート。間接的な依存も更新する |
| `github-actions` | `.github/workflows/`で使うAction |
| `rust-toolchain` | `rust-toolchain.toml`のRustの版。`Cargo.toml`の`rust-version`（MSRV）は更新しない |
| `pre-commit` | `.pre-commit-config.yaml`のGitのフックの版 |

Dependabotは、日本時間の毎週月曜日0時に新しいバージョンを確認し、更新があればPRを作成します。
脆弱性のある依存が見つかった場合は、週次の確認を待たずにセキュリティ更新のPRを作成します。
公開の直後に取り下げられた版や、乗っ取られて公開された版を取り込まないよう、バージョン更新では、公開から7日を経過した版のみを取り込みます（セキュリティ更新には適用されません）。

PRは次の単位でまとめて作成されます。

| 種類 | PRの単位 |
| --- | --- |
| クレートのminor・patch更新 | 1つのPRにまとめる |
| クレートのmajor更新(0.x系では`0.28`から`0.29`のような更新を含む) | クレートごとに1つのPR |
| Actionの更新 | 1つのPRにまとめる |
| セキュリティ更新 | `cargo`と`github-actions`のそれぞれで1つのPRにまとめる |

セキュリティ更新のPRを作成するには、リポジトリのSettings → Advanced Securityで次の設定を有効にする必要があります。

- Dependency graph
- Dependabot alerts
- Dependabot security updates

### PRへの対応

Dependabotが作成したPRには、次の手順で対応します。

1. CIのジョブがすべて成功していることを確認します。
2. PRの説明にあるリリースノートや変更履歴で、互換性のない変更(Breaking、Changedなど)がないかを確認します。
3. 互換性のない変更がある場合は、このリポジトリで使っている機能に影響するかを確認します。`dev-dependencies`のクレートは、このクレートの利用者には影響しません。
4. 問題がなければマージします。
5. マージ後、ローカルの`main`を最新にします。

CIが失敗した場合や、互換性のない変更に合わせてコードの修正が必要な場合は、PRのブランチで修正してからマージします。
修正せずに見送る場合は、PRにコメントで次のコマンドを書くと、Dependabotがその更新を作成しなくなります。

| コマンド | 効果 |
| --- | --- |
| `@dependabot ignore this major version` | このmajorバージョンへの更新を無視する |
| `@dependabot ignore this minor version` | このminorバージョンへの更新を無視する |
| `@dependabot ignore this dependency` | この依存の更新をすべて無視する |

PRがコンフリクトした場合は、`@dependabot rebase`とコメントするとDependabotがリベースします。
