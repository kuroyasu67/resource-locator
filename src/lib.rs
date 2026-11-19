//! リソースの位置を表す文字列を解析するクレート
//!
//! - ファイルの位置は[`FileLocation`]で表す(`gs://`、`local://`)。
//! - シークレットの位置は[`SecretLocation`]で表す(`none://`、`sm://`、`env://`)。
//!
//! 受け付ける形式の詳細は、それぞれの型のドキュメントを参照。

mod errors;
mod file_location;
mod scheme;
mod secret_location;

pub use errors::{Error, Result};
pub use file_location::{CloudStorageItem, FileLocation};
pub use secret_location::{EnvVarName, LATEST_VERSION, SecretLocation, SecretManagerItem};
