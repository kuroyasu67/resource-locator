//! ファイルロケーション

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::errors::{Error, Result};
use crate::scheme::{self, LOCAL_SCHEME, Parts};

/// Cloud Storageのスキーム
const CLOUD_STORAGE_SCHEME: &str = "gs";

/// [`FileLocation`]が受け付けるスキームの一覧(エラーメッセージ用)
const EXPECTED_SCHEMES: &str = "`gs://`, `local://`";

/// ドットを含まないバケット名の最大長
const MAX_BUCKET_NAME_LEN: usize = 63;

/// ドットを含むバケット名の最大長
const MAX_DOTTED_BUCKET_NAME_LEN: usize = 222;

/// バケット名の最小長
const MIN_BUCKET_NAME_LEN: usize = 3;

/// ファイルロケーション
///
/// 次のいずれかの形式の文字列から[`str::parse`]で構築する。
///
/// - `gs://<bucket-name>/<object-path>`: Cloud Storageのオブジェクト
/// - `local://<path>`: ローカルファイル
///
/// ```
/// use resource_locator::FileLocation;
///
/// let location: FileLocation = "gs://my-bucket/dir/config.toml".parse().unwrap();
/// let FileLocation::CloudStorage(item) = &location else { unreachable!() };
/// assert_eq!(item.bucket(), "my-bucket");
/// assert_eq!(item.path(), "dir/config.toml");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileLocation {
    /// Cloud Storageのファイル
    CloudStorage(CloudStorageItem),

    /// ローカルファイル
    Local(PathBuf),
}

impl FromStr for FileLocation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts = scheme::split(s)?;
        match parts.scheme {
            CLOUD_STORAGE_SCHEME => CloudStorageItem::from_parts(parts).map(Self::CloudStorage),
            LOCAL_SCHEME => parts.local_path().map(Self::Local),
            _ => Err(parts.unsupported(EXPECTED_SCHEMES)),
        }
    }
}

impl fmt::Display for FileLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CloudStorage(item) => item.fmt(f),
            Self::Local(path) => write!(f, "{LOCAL_SCHEME}://{}", path.display()),
        }
    }
}

/// `gs://`または`local://`で始まる文字列を、[`FromStr`]と同じ規則で解析する。
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FileLocation {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        crate::de::deserialize_from_str(
            deserializer,
            "a file location such as `gs://...` or `local://...`",
        )
    }
}

impl FileLocation {
    /// ローカルファイルの場合はそのパスを返す。
    #[must_use]
    pub fn as_local(&self) -> Option<&Path> {
        match self {
            Self::Local(path) => Some(path),
            Self::CloudStorage(_) => None,
        }
    }

    /// Cloud Storageの場合はそのアイテムを返す。
    #[must_use]
    pub fn as_cloud_storage(&self) -> Option<&CloudStorageItem> {
        match self {
            Self::CloudStorage(item) => Some(item),
            Self::Local(_) => None,
        }
    }
}

/// Cloud Storageのアイテム
///
/// バケット名は検証済みで、オブジェクトパスは空にならない。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CloudStorageItem {
    /// バケット名
    bucket: String,

    /// オブジェクトパス
    path: String,
}

impl CloudStorageItem {
    /// バケット名を返す。
    #[must_use]
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// オブジェクトパスを返す。
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    fn from_parts(parts: Parts<'_>) -> Result<Self> {
        let location = parts.location;
        let (bucket, path) = match parts.rest.split_once('/') {
            Some((bucket, path)) => (bucket.trim(), path.trim()),
            None => (parts.rest.trim(), ""),
        };
        if bucket.is_empty() {
            return Err(Error::MissingBucket {
                location: location.to_owned(),
            });
        }
        validate_bucket_name(bucket).map_err(|reason| Error::InvalidBucketName {
            location: location.to_owned(),
            bucket: bucket.to_owned(),
            reason,
        })?;
        if path.is_empty() {
            return Err(Error::MissingObjectPath {
                location: location.to_owned(),
            });
        }
        Ok(Self {
            bucket: bucket.to_owned(),
            path: path.to_owned(),
        })
    }
}

impl FromStr for CloudStorageItem {
    type Err = Error;

    /// `gs://<bucket-name>/<object-path>`形式の文字列を解析する。
    fn from_str(s: &str) -> Result<Self> {
        let parts = scheme::split(s)?;
        if parts.scheme != CLOUD_STORAGE_SCHEME {
            return Err(parts.unsupported("`gs://`"));
        }
        Self::from_parts(parts)
    }
}

impl fmt::Display for CloudStorageItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{CLOUD_STORAGE_SCHEME}://{}/{}", self.bucket, self.path)
    }
}

/// バケット名がCloud Storageの命名規則に従うか検証する。
///
/// - 英小文字、数字、`-`、`_`、`.`のみ
/// - 先頭と末尾は英小文字または数字
/// - 3〜63文字(`.`を含む場合は全体で222文字以内、`.`で区切った各部分は63文字以内)
fn validate_bucket_name(bucket: &str) -> std::result::Result<(), &'static str> {
    let is_valid_byte =
        |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_' | b'.');
    if !bucket.bytes().all(is_valid_byte) {
        return Err("must contain only lowercase letters, digits, `-`, `_` and `.`");
    }
    let is_lower_alphanumeric =
        |b: Option<u8>| b.is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
    if !is_lower_alphanumeric(bucket.bytes().next())
        || !is_lower_alphanumeric(bucket.bytes().last())
    {
        return Err("must start and end with a lowercase letter or digit");
    }
    if bucket.len() < MIN_BUCKET_NAME_LEN {
        return Err("must be at least 3 characters");
    }
    if bucket.contains('.') {
        if bucket.len() > MAX_DOTTED_BUCKET_NAME_LEN {
            return Err("must be at most 222 characters");
        }
        if bucket.split('.').any(|c| c.len() > MAX_BUCKET_NAME_LEN) {
            return Err("each dot-separated component must be at most 63 characters");
        }
    } else if bucket.len() > MAX_BUCKET_NAME_LEN {
        return Err("must be at most 63 characters");
    }
    Ok(())
}

#[cfg(test)]
mod file_location_tests {
    use rstest::rstest;

    use super::*;

    fn gs(bucket: &str, path: &str) -> FileLocation {
        FileLocation::CloudStorage(CloudStorageItem {
            bucket: bucket.to_owned(),
            path: path.to_owned(),
        })
    }

    fn local(path: &str) -> FileLocation {
        FileLocation::Local(PathBuf::from(path))
    }

    #[rstest]
    #[case::gs("gs://my-bucket/config.toml", gs("my-bucket", "config.toml"))]
    #[case::gs_nested_path(
        "gs://my-bucket/dir/sub/config.toml",
        gs("my-bucket", "dir/sub/config.toml")
    )]
    #[case::gs_dotted_bucket("gs://example.com/config.toml", gs("example.com", "config.toml"))]
    #[case::gs_trims_bucket_and_path(
        "gs:// my-bucket / config.toml ",
        gs("my-bucket", "config.toml")
    )]
    #[case::gs_trims_location("  gs://my-bucket/config.toml  ", gs("my-bucket", "config.toml"))]
    #[case::local_relative("local://./config.toml", local("./config.toml"))]
    #[case::local_absolute("local:///etc/app/config.toml", local("/etc/app/config.toml"))]
    #[case::local_file_name("local://config.toml", local("config.toml"))]
    #[case::local_trims_path("local://  config.toml  ", local("config.toml"))]
    #[case::local_trims_location("  local://config.toml  ", local("config.toml"))]
    fn parse_returns_file_location(#[case] location: &str, #[case] expected: FileLocation) {
        assert_eq!(location.parse::<FileLocation>(), Ok(expected));
    }

    #[rstest]
    #[case::empty("", Error::Empty)]
    #[case::blank("   ", Error::Empty)]
    #[case::no_scheme(
        "my-bucket/config.toml",
        Error::MissingScheme { location: "my-bucket/config.toml".into() }
    )]
    #[case::bare_path(
        "/etc/app/config.toml",
        Error::MissingScheme { location: "/etc/app/config.toml".into() }
    )]
    #[case::malformed_scheme(
        "gs:/my-bucket/config.toml",
        Error::MissingScheme { location: "gs:/my-bucket/config.toml".into() }
    )]
    #[case::unknown_scheme(
        "s3://my-bucket/config.toml",
        Error::UnsupportedScheme {
            location: "s3://my-bucket/config.toml".into(),
            scheme: "s3".into(),
            expected: EXPECTED_SCHEMES,
        }
    )]
    #[case::uppercase_scheme(
        "GS://my-bucket/config.toml",
        Error::UnsupportedScheme {
            location: "GS://my-bucket/config.toml".into(),
            scheme: "GS".into(),
            expected: EXPECTED_SCHEMES,
        }
    )]
    #[case::gs_empty("gs://", Error::MissingBucket { location: "gs://".into() })]
    #[case::gs_empty_bucket(
        "gs:///config.toml",
        Error::MissingBucket { location: "gs:///config.toml".into() }
    )]
    #[case::gs_blank_bucket(
        "gs://  /config.toml",
        Error::MissingBucket { location: "gs://  /config.toml".into() }
    )]
    #[case::gs_bucket_only("gs://my-bucket", Error::MissingObjectPath { location: "gs://my-bucket".into() })]
    #[case::gs_empty_path(
        "gs://my-bucket/",
        Error::MissingObjectPath { location: "gs://my-bucket/".into() }
    )]
    #[case::gs_blank_path(
        "gs://my-bucket/   ",
        Error::MissingObjectPath { location: "gs://my-bucket/".into() }
    )]
    #[case::local_empty("local://", Error::MissingLocalPath { location: "local://".into() })]
    #[case::local_blank("local://   ", Error::MissingLocalPath { location: "local://".into() })]
    fn parse_rejects_invalid_location(#[case] location: &str, #[case] expected: Error) {
        assert_eq!(location.parse::<FileLocation>(), Err(expected));
    }

    #[rstest]
    #[case::uppercase(
        "My-Bucket",
        "must contain only lowercase letters, digits, `-`, `_` and `.`"
    )]
    #[case::invalid_char(
        "my$bucket",
        "must contain only lowercase letters, digits, `-`, `_` and `.`"
    )]
    #[case::starts_with_hyphen("-bucket", "must start and end with a lowercase letter or digit")]
    #[case::ends_with_dot("bucket.", "must start and end with a lowercase letter or digit")]
    #[case::too_short("ab", "must be at least 3 characters")]
    #[case::too_long(&"a".repeat(64), "must be at most 63 characters")]
    #[case::dotted_too_long(
        &format!("{}.{}.{}.{}", "a".repeat(55), "a".repeat(55), "a".repeat(55), "a".repeat(55)),
        "must be at most 222 characters"
    )]
    #[case::dotted_component_too_long(
        &format!("{}.com", "a".repeat(64)),
        "each dot-separated component must be at most 63 characters"
    )]
    fn parse_rejects_invalid_bucket_name(#[case] bucket: &str, #[case] reason: &'static str) {
        let location = format!("gs://{bucket}/config.toml");
        assert_eq!(
            location.parse::<FileLocation>(),
            Err(Error::InvalidBucketName {
                location: location.clone(),
                bucket: bucket.to_owned(),
                reason,
            })
        );
    }

    #[rstest]
    #[case::min_length("abc")]
    #[case::max_length(&"a".repeat(63))]
    #[case::with_underscore("my_bucket")]
    #[case::dotted_max_component(&format!("{}.com", "a".repeat(63)))]
    fn parse_accepts_boundary_bucket_name(#[case] bucket: &str) {
        let location = format!("gs://{bucket}/config.toml");
        assert_eq!(
            location.parse::<FileLocation>(),
            Ok(gs(bucket, "config.toml"))
        );
    }

    #[rstest]
    #[case::gs("gs:// my-bucket / dir/config.toml ", "gs://my-bucket/dir/config.toml")]
    #[case::local("local:// /etc/app.toml ", "local:///etc/app.toml")]
    fn display_returns_normalized_location(#[case] location: &str, #[case] expected: &str) {
        let parsed: FileLocation = location.parse().unwrap();
        assert_eq!(parsed.to_string(), expected);
        assert_eq!(expected.parse::<FileLocation>(), Ok(parsed));
    }

    #[cfg(feature = "serde")]
    #[rstest]
    #[case::gs(" gs://my-bucket/config.toml ", Ok(gs("my-bucket", "config.toml")))]
    #[case::local("local:///etc/app.toml", Ok(local("/etc/app.toml")))]
    #[case::invalid(
        "s3://my-bucket/config.toml",
        Err(
            "unsupported scheme `s3` in `s3://my-bucket/config.toml` (expected one of: `gs://`, `local://`)"
        )
    )]
    fn deserialize_parses_string(
        #[case] location: &str,
        #[case] expected: std::result::Result<FileLocation, &str>,
    ) {
        use serde::Deserialize;
        use serde::de::IntoDeserializer;
        use serde::de::value::{Error as ValueError, StrDeserializer};

        let deserializer: StrDeserializer<'_, ValueError> = location.into_deserializer();
        let actual = FileLocation::deserialize(deserializer).map_err(|e| e.to_string());
        assert_eq!(actual, expected.map_err(str::to_owned));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_rejects_non_string() {
        use serde::Deserialize;
        use serde::de::IntoDeserializer;
        use serde::de::value::{Error as ValueError, U32Deserializer};

        let deserializer: U32Deserializer<ValueError> = 42_u32.into_deserializer();
        let err = FileLocation::deserialize(deserializer).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid type: integer `42`, expected a file location such as `gs://...` or `local://...`"
        );
    }

    #[test]
    fn accessors_return_matching_variant() {
        let gs_location = gs("my-bucket", "config.toml");
        assert!(gs_location.as_local().is_none());
        assert_eq!(
            gs_location.as_cloud_storage().map(CloudStorageItem::bucket),
            Some("my-bucket")
        );

        let local_location = local("/etc/app.toml");
        assert!(local_location.as_cloud_storage().is_none());
        assert_eq!(local_location.as_local(), Some(Path::new("/etc/app.toml")));
    }
}

#[cfg(test)]
mod cloud_storage_item_tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::ok(
        "gs://my-bucket/dir/config.toml",
        Ok(CloudStorageItem { bucket: "my-bucket".into(), path: "dir/config.toml".into() })
    )]
    #[case::trims("  gs:// my-bucket / config.toml  ", Ok(CloudStorageItem { bucket: "my-bucket".into(), path: "config.toml".into() }))]
    #[case::without_scheme(
        "my-bucket/config.toml",
        Err(Error::MissingScheme { location: "my-bucket/config.toml".into() })
    )]
    #[case::other_scheme(
        "local://config.toml",
        Err(Error::UnsupportedScheme {
            location: "local://config.toml".into(),
            scheme: "local".into(),
            expected: "`gs://`",
        })
    )]
    fn parse(#[case] location: &str, #[case] expected: Result<CloudStorageItem>) {
        assert_eq!(location.parse::<CloudStorageItem>(), expected);
    }
}
