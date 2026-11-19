# resource-locator

設定ファイルや環境変数に書いた「リソースの場所」を表す文字列を解析し、型付きの値に変換するRustクレートです。

- ファイルの場所: Cloud Storageのオブジェクト、またはローカルファイル
- シークレットの場所: Secret Managerのシークレット、環境変数、またはシークレットなし

このクレートは文字列の解析と検証だけを行います。ファイルやシークレットの中身を実際に読み込む処理は含みません。

## 使い方

`Cargo.toml`に追加します。

```toml
[dependencies]
resource-locator = { git = "https://github.com/kuroyasu67/resource-locator" }
```

文字列を`parse`すると、場所の種類に応じた値が得られます。

```rust
use resource_locator::{FileLocation, SecretLocation};

let config: FileLocation = "gs://my-bucket/app/config.toml".parse()?;
match &config {
    FileLocation::CloudStorage(item) => {
        assert_eq!(item.bucket(), "my-bucket");
        assert_eq!(item.path(), "app/config.toml");
    }
    FileLocation::Local(path) => println!("read {}", path.display()),
}

let password: SecretLocation = "sm://my-project/db-password/3".parse()?;
match &password {
    SecretLocation::SecretManager(item) => {
        assert_eq!(item.project(), Some("my-project"));
        assert_eq!(item.secret(), "db-password");
        assert_eq!(item.version(), "3");
    }
    SecretLocation::Env(name) => println!("read ${}", name.as_str()),
    SecretLocation::None => println!("no secret"),
}
# Ok::<(), resource_locator::Error>(())
```

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

```rust
use resource_locator::{Error, SecretLocation};

let err = "env://1API_KEY".parse::<SecretLocation>().unwrap_err();
assert!(matches!(
    err,
    Error::InvalidEnvVarName { reason: "must not start with a digit", .. }
));
assert_eq!(
    err.to_string(),
    "invalid environment variable name `1API_KEY` in `env://1API_KEY`: must not start with a digit",
);
```

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
| `MissingLocalPath` | ローカルパスが空 |

`Error`には`#[non_exhaustive]`が付いているため、`match`するときはワイルドカード(`_`)の腕が必要です。

## 文字列への変換

どの型も`Display`を実装しており、正規化した形式で出力します。
出力した文字列を再び解析すると、元と同じ値になります。

```rust
use resource_locator::SecretLocation;

let location: SecretLocation = "sm://projects/my-project/secrets/db-password".parse()?;
assert_eq!(location.to_string(), "sm://my-project/db-password/latest");
# Ok::<(), resource_locator::Error>(())
```

## serdeによるデシリアライズ

`serde`フィーチャーを有効にすると、`FileLocation`と`SecretLocation`が`serde::Deserialize`を実装します。
文字列を`parse`と同じ規則で解析するため、設定ファイルの構造体のフィールドにそのまま使えます。

```toml
[dependencies]
resource-locator = { git = "https://github.com/kuroyasu67/resource-locator", features = ["serde"] }
```

```rust,ignore
use resource_locator::{FileLocation, SecretLocation};

#[derive(serde::Deserialize)]
struct Config {
    template: FileLocation,
    api_key: SecretLocation,
}
```

解析に失敗した場合は、`Error`のメッセージがデシリアライザーのエラーとして返ります。

## 動作環境

Rust 1.100.0以降
