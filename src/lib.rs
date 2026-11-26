//! リソースの位置を表す文字列を解析するクレート
//!
//! - ファイルの位置は[`FileLocation`]で表す(`gs://`、`local://`)。
//! - シークレットの位置は[`SecretLocation`]で表す(`none://`、`sm://`、`env://`、`local://`)。
//!
//! 受け付ける形式の詳細は、それぞれの型のドキュメントを参照。
//!
//! 文字列を[`str::parse`]で解析すると、場所の種類に応じた値が得られる。
//!
//! ```
//! use resource_locator::{FileLocation, SecretLocation};
//!
//! let config: FileLocation = "gs://my-bucket/app/config.toml".parse()?;
//! match &config {
//!     FileLocation::CloudStorage(item) => {
//!         assert_eq!(item.bucket(), "my-bucket");
//!         assert_eq!(item.path(), "app/config.toml");
//!     }
//!     FileLocation::Local(path) => println!("read {}", path.display()),
//! }
//!
//! let password: SecretLocation = "sm://my-project/db-password/3".parse()?;
//! match &password {
//!     SecretLocation::SecretManager(item) => {
//!         assert_eq!(item.project(), Some("my-project"));
//!         assert_eq!(item.secret(), "db-password");
//!         assert_eq!(item.version(), "3");
//!     }
//!     SecretLocation::Env(name) => println!("read ${}", name.as_str()),
//!     SecretLocation::Local(path) => println!("read {}", path.display()),
//!     SecretLocation::None => println!("no secret"),
//! }
//! # Ok::<(), resource_locator::Error>(())
//! ```

#[cfg(feature = "serde")]
mod de;
mod errors;
mod file_location;
mod scheme;
mod secret_location;

pub use errors::{Error, Result};
pub use file_location::{CloudStorageItem, FileLocation};
pub use secret_location::{EnvVarName, LATEST_VERSION, SecretLocation, SecretManagerItem};
