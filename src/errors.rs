//! 結果とエラー型

/// エラー型
///
/// どのバリアントも、原因を特定できるように解析対象の文字列全体を`location`に保持する。
///
/// ```
/// use resource_locator::{Error, SecretLocation};
///
/// let err = "env://1API_KEY".parse::<SecretLocation>().unwrap_err();
/// assert!(matches!(
///     err,
///     Error::InvalidEnvVarName { reason: "must not start with a digit", .. }
/// ));
/// assert_eq!(
///     err.to_string(),
///     "invalid environment variable name `1API_KEY` in `env://1API_KEY`: must not start with a digit",
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// ロケーションが空、または空白のみ
    #[error("resource location is empty")]
    Empty,

    /// `<scheme>://`の形式になっていない
    #[error("resource location `{location}` has no scheme (expected `<scheme>://...`)")]
    MissingScheme {
        /// 解析対象の文字列
        location: String,
    },

    /// 対応していないスキーム
    #[error("unsupported scheme `{scheme}` in `{location}` (expected one of: {expected})")]
    UnsupportedScheme {
        /// 解析対象の文字列
        location: String,
        /// 指定されたスキーム
        scheme: String,
        /// 受け付けるスキームの一覧
        expected: &'static str,
    },

    /// 後ろに何も続かないはずのスキーム(`none://`)の後ろに文字列がある
    #[error("`{scheme}://` must not be followed by anything, but got `{location}`")]
    UnexpectedContent {
        /// 解析対象の文字列
        location: String,
        /// スキーム
        scheme: &'static str,
    },

    /// Secret Managerのパスの構造が、受け付けるどの形式にも一致しない
    #[error("malformed secret manager path in `{location}`: {reason}")]
    MalformedSecretPath {
        /// 解析対象の文字列
        location: String,
        /// 不正な理由
        reason: &'static str,
    },

    /// Secret Managerのプロジェクトが空
    #[error("project is missing in `{location}`")]
    MissingProject {
        /// 解析対象の文字列
        location: String,
    },

    /// Secret Managerのプロジェクトが不正
    #[error("invalid project `{project}` in `{location}`: {reason}")]
    InvalidProject {
        /// 解析対象の文字列
        location: String,
        /// プロジェクトIDまたはプロジェクト番号
        project: String,
        /// 不正な理由
        reason: &'static str,
    },

    /// Secret Managerのシークレット名がない
    #[error("secret name is missing in `{location}`")]
    MissingSecretName {
        /// 解析対象の文字列
        location: String,
    },

    /// Secret Managerのシークレット名が不正
    #[error("invalid secret name `{name}` in `{location}`: {reason}")]
    InvalidSecretName {
        /// 解析対象の文字列
        location: String,
        /// シークレット名
        name: String,
        /// 不正な理由
        reason: &'static str,
    },

    /// Secret Managerのバージョンが不正
    #[error("invalid secret version `{version}` in `{location}`: {reason}")]
    InvalidSecretVersion {
        /// 解析対象の文字列
        location: String,
        /// バージョン
        version: String,
        /// 不正な理由
        reason: &'static str,
    },

    /// 環境変数名がない
    #[error("environment variable name is missing in `{location}`")]
    MissingEnvVarName {
        /// 解析対象の文字列
        location: String,
    },

    /// 環境変数名が不正
    #[error("invalid environment variable name `{name}` in `{location}`: {reason}")]
    InvalidEnvVarName {
        /// 解析対象の文字列
        location: String,
        /// 環境変数名
        name: String,
        /// 不正な理由
        reason: &'static str,
    },

    /// Cloud Storageのバケット名がない
    #[error("bucket name is missing in `{location}`")]
    MissingBucket {
        /// 解析対象の文字列
        location: String,
    },

    /// Cloud Storageのバケット名が不正
    #[error("invalid bucket name `{bucket}` in `{location}`: {reason}")]
    InvalidBucketName {
        /// 解析対象の文字列
        location: String,
        /// バケット名
        bucket: String,
        /// 不正な理由
        reason: &'static str,
    },

    /// Cloud Storageのオブジェクトパスがない
    #[error("object path is missing in `{location}`")]
    MissingObjectPath {
        /// 解析対象の文字列
        location: String,
    },

    /// ローカルファイルのパスがない
    #[error("local file path is missing in `{location}`")]
    MissingLocalPath {
        /// 解析対象の文字列
        location: String,
    },
}

/// 結果型
pub type Result<T> = std::result::Result<T, Error>;
