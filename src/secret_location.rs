//! シークレットロケーション

use std::fmt;
use std::str::FromStr;

use crate::errors::{Error, Result};
use crate::scheme::{self, Parts};

/// シークレットがないことを示すスキーム
const NONE_SCHEME: &str = "none";

/// Secret Managerのスキーム
const SECRET_MANAGER_SCHEME: &str = "sm";

/// 環境変数のスキーム
const ENV_SCHEME: &str = "env";

/// [`SecretLocation`]が受け付けるスキームの一覧(エラーメッセージ用)
const EXPECTED_SCHEMES: &str = "`none://`, `sm://`, `env://`";

/// バージョンを省略したときに使用するバージョン
pub const LATEST_VERSION: &str = "latest";

/// シークレット名の最大長(Secret Managerの制約)
const MAX_SECRET_NAME_LEN: usize = 255;

/// プロジェクトIDの最小長
const MIN_PROJECT_ID_LEN: usize = 6;

/// プロジェクトIDの最大長
const MAX_PROJECT_ID_LEN: usize = 30;

/// Secret Managerのパスが受け付けるどの形式にも一致しないときの理由
const SECRET_PATH_FORMATS: &str = "expected `<secret>`, `<secret>/<version>`, \
    `<project>/<secret>/<version>` or `projects/<project>/secrets/<secret>[/versions/<version>]`";

/// シークレットロケーション
///
/// 次のいずれかの形式の文字列から[`str::parse`]で構築する。
///
/// - `none://`: シークレットがない
/// - `env://<variable-name>`: 環境変数
/// - Secret Managerのシークレット(`<version>`を省略した場合は`latest`)
///   - `sm://<secret>`
///   - `sm://<secret>/<version>`
///   - `sm://<project>/<secret>/<version>`
///   - `sm://projects/<project>/secrets/<secret>`
///   - `sm://projects/<project>/secrets/<secret>/versions/<version>`
///
/// `/`で区切った部分の数で形式を判別する。
/// プロジェクトを指定してバージョンを省略するには、`sm://<project>/<secret>/latest`、
/// `sm://<project>/<secret>/`、または`sm://projects/<project>/secrets/<secret>`と書く。
/// `<project>`にはプロジェクトIDとプロジェクト番号のどちらも指定できる。
///
/// ```
/// use resource_locator::SecretLocation;
///
/// let location: SecretLocation = "sm://my-project/db-password/3".parse().unwrap();
/// let SecretLocation::SecretManager(item) = &location else { unreachable!() };
/// assert_eq!(item.project(), Some("my-project"));
/// assert_eq!(item.secret(), "db-password");
/// assert_eq!(item.version(), "3");
///
/// let location: SecretLocation = "sm://db-password".parse().unwrap();
/// let SecretLocation::SecretManager(item) = &location else { unreachable!() };
/// assert_eq!(item.project(), None);
/// assert_eq!(item.version(), "latest");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecretLocation {
    /// シークレットがない
    None,

    /// Secret Managerにシークレットを保持
    SecretManager(SecretManagerItem),

    /// 環境変数にシークレットを保持
    Env(EnvVarName),
}

impl FromStr for SecretLocation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts = scheme::split(s)?;
        match parts.scheme {
            NONE_SCHEME if parts.rest.is_empty() => Ok(Self::None),
            NONE_SCHEME => Err(Error::UnexpectedContent {
                location: parts.location.to_owned(),
                scheme: NONE_SCHEME,
            }),
            SECRET_MANAGER_SCHEME => SecretManagerItem::from_parts(parts).map(Self::SecretManager),
            ENV_SCHEME => EnvVarName::from_parts(parts).map(Self::Env),
            _ => Err(parts.unsupported(EXPECTED_SCHEMES)),
        }
    }
}

impl fmt::Display for SecretLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "{NONE_SCHEME}://"),
            Self::SecretManager(item) => item.fmt(f),
            Self::Env(name) => name.fmt(f),
        }
    }
}

/// Secret Managerのアイテム
///
/// プロジェクト、シークレット名、バージョンは検証済みで、空になることはない。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretManagerItem {
    /// プロジェクトIDまたはプロジェクト番号(省略された場合は`None`)
    project: Option<String>,

    /// シークレット名
    secret: String,

    /// バージョン
    version: String,
}

impl SecretManagerItem {
    /// プロジェクトIDまたはプロジェクト番号を返す。
    ///
    /// 省略された場合は`None`を返す。
    /// この場合、どのプロジェクトのシークレットを読むかは利用側が決める。
    #[must_use]
    pub fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }

    /// シークレット名を返す。
    #[must_use]
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// バージョンを返す。省略された場合は[`LATEST_VERSION`]。
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    fn from_parts(parts: Parts<'_>) -> Result<Self> {
        let location = parts.location;
        let segments: Vec<&str> = parts.rest.split('/').map(str::trim).collect();
        let (project, secret, version) = match segments.as_slice() {
            ["projects", project, "secrets", secret] => (Some(*project), *secret, ""),
            ["projects", project, "secrets", secret, "versions", version] => {
                (Some(*project), *secret, *version)
            }
            [secret] => (None, *secret, ""),
            [secret, version] => (None, *secret, *version),
            [project, secret, version] => (Some(*project), *secret, *version),
            _ => {
                return Err(Error::MalformedSecretPath {
                    location: location.to_owned(),
                    reason: SECRET_PATH_FORMATS,
                });
            }
        };
        if let Some(project) = project {
            if project.is_empty() {
                return Err(Error::MissingProject {
                    location: location.to_owned(),
                });
            }
            validate_project(project).map_err(|reason| Error::InvalidProject {
                location: location.to_owned(),
                project: project.to_owned(),
                reason,
            })?;
        }
        if secret.is_empty() {
            return Err(Error::MissingSecretName {
                location: location.to_owned(),
            });
        }
        validate_secret_name(secret).map_err(|reason| Error::InvalidSecretName {
            location: location.to_owned(),
            name: secret.to_owned(),
            reason,
        })?;
        let version = if version.is_empty() {
            LATEST_VERSION
        } else {
            validate_secret_version(version).map_err(|reason| Error::InvalidSecretVersion {
                location: location.to_owned(),
                version: version.to_owned(),
                reason,
            })?;
            version
        };
        Ok(Self {
            project: project.map(str::to_owned),
            secret: secret.to_owned(),
            version: version.to_owned(),
        })
    }
}

impl FromStr for SecretManagerItem {
    type Err = Error;

    /// `sm://`で始まる文字列を解析する。受け付ける形式は[`SecretLocation`]を参照。
    fn from_str(s: &str) -> Result<Self> {
        let parts = scheme::split(s)?;
        if parts.scheme != SECRET_MANAGER_SCHEME {
            return Err(parts.unsupported("`sm://`"));
        }
        Self::from_parts(parts)
    }
}

impl fmt::Display for SecretManagerItem {
    /// `sm://<secret>/<version>`、またはプロジェクトがある場合は
    /// `sm://<project>/<secret>/<version>`の形式で出力する。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{SECRET_MANAGER_SCHEME}://")?;
        if let Some(project) = &self.project {
            write!(f, "{project}/")?;
        }
        write!(f, "{}/{}", self.secret, self.version)
    }
}

/// 環境変数名
///
/// POSIXの環境変数名の規則(`[A-Za-z_][A-Za-z0-9_]*`)に従うことが保証されている。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnvVarName(String);

impl EnvVarName {
    /// 環境変数名を返す。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn from_parts(parts: Parts<'_>) -> Result<Self> {
        let name = parts.rest.trim();
        if name.is_empty() {
            return Err(Error::MissingEnvVarName {
                location: parts.location.to_owned(),
            });
        }
        validate_env_var_name(name).map_err(|reason| Error::InvalidEnvVarName {
            location: parts.location.to_owned(),
            name: name.to_owned(),
            reason,
        })?;
        Ok(Self(name.to_owned()))
    }
}

/// 環境変数名がPOSIXの規則(`[A-Za-z_][A-Za-z0-9_]*`)に従うか検証する。
fn validate_env_var_name(name: &str) -> std::result::Result<(), &'static str> {
    if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err("must contain only ASCII letters, digits and `_`");
    }
    if name.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Err("must not start with a digit");
    }
    Ok(())
}

impl AsRef<str> for EnvVarName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EnvVarName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{ENV_SCHEME}://{}", self.0)
    }
}

/// シークレット名がSecret Managerの命名規則(英数字、`-`、`_`のみ、255文字以内)に従うか検証する。
fn validate_secret_name(name: &str) -> std::result::Result<(), &'static str> {
    if name.len() > MAX_SECRET_NAME_LEN {
        return Err("must be at most 255 characters");
    }
    if !name.bytes().all(is_secret_id_byte) {
        return Err("must contain only ASCII letters, digits, `-` and `_`");
    }
    Ok(())
}

/// バージョンが、番号、`latest`、またはエイリアスとして有効な文字だけで構成されているか検証する。
fn validate_secret_version(version: &str) -> std::result::Result<(), &'static str> {
    if !version.bytes().all(is_secret_id_byte) {
        return Err("must contain only ASCII letters, digits, `-` and `_`");
    }
    Ok(())
}

/// プロジェクトを検証する。
///
/// 数字のみの場合はプロジェクト番号とみなす。それ以外はプロジェクトIDとみなし、次の規則に従うか検証する。
///
/// - 英小文字、数字、`-`のみ
/// - 先頭は英小文字、末尾は`-`以外
/// - 6〜30文字
fn validate_project(project: &str) -> std::result::Result<(), &'static str> {
    if project.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(());
    }
    if !project
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("must contain only lowercase letters, digits and `-`");
    }
    if !project.as_bytes()[0].is_ascii_lowercase() {
        return Err("must start with a lowercase letter");
    }
    if project.ends_with('-') {
        return Err("must not end with `-`");
    }
    if !(MIN_PROJECT_ID_LEN..=MAX_PROJECT_ID_LEN).contains(&project.len()) {
        return Err("must be 6 to 30 characters");
    }
    Ok(())
}

fn is_secret_id_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

#[cfg(test)]
mod secret_location_tests {
    use rstest::rstest;

    use super::*;

    fn item(project: Option<&str>, secret: &str, version: &str) -> SecretManagerItem {
        SecretManagerItem {
            project: project.map(str::to_owned),
            secret: secret.to_owned(),
            version: version.to_owned(),
        }
    }

    fn sm(secret: &str, version: &str) -> SecretLocation {
        SecretLocation::SecretManager(item(None, secret, version))
    }

    fn sm_in(project: &str, secret: &str, version: &str) -> SecretLocation {
        SecretLocation::SecretManager(item(Some(project), secret, version))
    }

    fn env(name: &str) -> SecretLocation {
        SecretLocation::Env(EnvVarName(name.to_owned()))
    }

    #[rstest]
    #[case::none("none://", SecretLocation::None)]
    #[case::none_trims_location("  none://  ", SecretLocation::None)]
    #[case::sm_without_version("sm://my-secret", sm("my-secret", "latest"))]
    #[case::sm_with_version("sm://my-secret/3", sm("my-secret", "3"))]
    #[case::sm_with_latest("sm://my-secret/latest", sm("my-secret", "latest"))]
    #[case::sm_with_alias("sm://my_secret/prod-v1", sm("my_secret", "prod-v1"))]
    #[case::sm_with_empty_version("sm://my-secret/", sm("my-secret", "latest"))]
    #[case::sm_with_blank_version("sm://my-secret/   ", sm("my-secret", "latest"))]
    #[case::sm_trims_secret("sm://  my-secret  ", sm("my-secret", "latest"))]
    #[case::sm_trims_secret_and_version("sm:// my-secret / 5 ", sm("my-secret", "5"))]
    #[case::sm_secret_named_projects("sm://projects/1", sm("projects", "1"))]
    #[case::sm_with_project("sm://my-project/my-secret/3", sm_in("my-project", "my-secret", "3"))]
    #[case::sm_with_project_number(
        "sm://123456789012/my-secret/3",
        sm_in("123456789012", "my-secret", "3")
    )]
    #[case::sm_with_project_and_empty_version(
        "sm://my-project/my-secret/",
        sm_in("my-project", "my-secret", "latest")
    )]
    #[case::sm_with_project_trims(
        "sm:// my-project / my-secret / 3 ",
        sm_in("my-project", "my-secret", "3")
    )]
    #[case::sm_resource_name(
        "sm://projects/my-project/secrets/my-secret/versions/3",
        sm_in("my-project", "my-secret", "3")
    )]
    #[case::sm_resource_name_without_version(
        "sm://projects/my-project/secrets/my-secret",
        sm_in("my-project", "my-secret", "latest")
    )]
    #[case::sm_resource_name_with_project_number(
        "sm://projects/123456789012/secrets/my-secret/versions/latest",
        sm_in("123456789012", "my-secret", "latest")
    )]
    #[case::env("env://DATABASE_PASSWORD", env("DATABASE_PASSWORD"))]
    #[case::env_trims_name("env://  API_KEY  ", env("API_KEY"))]
    fn parse_returns_secret_location(#[case] location: &str, #[case] expected: SecretLocation) {
        assert_eq!(location.parse::<SecretLocation>(), Ok(expected));
    }

    #[rstest]
    #[case::empty("", Error::Empty)]
    #[case::blank("   ", Error::Empty)]
    #[case::no_scheme("my-secret", Error::MissingScheme { location: "my-secret".into() })]
    #[case::malformed_scheme(
        "sm:/my-secret",
        Error::MissingScheme { location: "sm:/my-secret".into() }
    )]
    #[case::unknown_scheme(
        "foo://bar",
        Error::UnsupportedScheme {
            location: "foo://bar".into(),
            scheme: "foo".into(),
            expected: EXPECTED_SCHEMES,
        }
    )]
    #[case::uppercase_scheme(
        "SM://my-secret",
        Error::UnsupportedScheme {
            location: "SM://my-secret".into(),
            scheme: "SM".into(),
            expected: EXPECTED_SCHEMES,
        }
    )]
    #[case::none_with_trailing_text(
        "none://something",
        Error::UnexpectedContent { location: "none://something".into(), scheme: "none" }
    )]
    #[case::sm_empty("sm://", Error::MissingSecretName { location: "sm://".into() })]
    #[case::sm_blank("sm://   ", Error::MissingSecretName { location: "sm://".into() })]
    #[case::sm_empty_secret(
        "sm:///latest",
        Error::MissingSecretName { location: "sm:///latest".into() }
    )]
    #[case::sm_blank_secret("sm://  /1", Error::MissingSecretName { location: "sm://  /1".into() })]
    #[case::sm_invalid_secret_name(
        "sm://my.secret",
        Error::InvalidSecretName {
            location: "sm://my.secret".into(),
            name: "my.secret".into(),
            reason: "must contain only ASCII letters, digits, `-` and `_`",
        }
    )]
    #[case::sm_invalid_version(
        "sm://my-secret/v.1",
        Error::InvalidSecretVersion {
            location: "sm://my-secret/v.1".into(),
            version: "v.1".into(),
            reason: "must contain only ASCII letters, digits, `-` and `_`",
        }
    )]
    #[case::sm_four_segments(
        "sm://a/b/c/d",
        Error::MalformedSecretPath { location: "sm://a/b/c/d".into(), reason: SECRET_PATH_FORMATS }
    )]
    #[case::sm_resource_name_wrong_keyword(
        "sm://projects/my-project/keys/my-secret",
        Error::MalformedSecretPath {
            location: "sm://projects/my-project/keys/my-secret".into(),
            reason: SECRET_PATH_FORMATS,
        }
    )]
    #[case::sm_resource_name_missing_versions_keyword(
        "sm://projects/my-project/secrets/my-secret/3",
        Error::MalformedSecretPath {
            location: "sm://projects/my-project/secrets/my-secret/3".into(),
            reason: SECRET_PATH_FORMATS,
        }
    )]
    #[case::sm_resource_name_too_long(
        "sm://projects/my-project/secrets/my-secret/versions/3/extra",
        Error::MalformedSecretPath {
            location: "sm://projects/my-project/secrets/my-secret/versions/3/extra".into(),
            reason: SECRET_PATH_FORMATS,
        }
    )]
    #[case::sm_empty_project(
        "sm:///my-secret/1",
        Error::MissingProject { location: "sm:///my-secret/1".into() }
    )]
    #[case::sm_resource_name_empty_project(
        "sm://projects//secrets/my-secret",
        Error::MissingProject { location: "sm://projects//secrets/my-secret".into() }
    )]
    #[case::sm_with_project_empty_secret(
        "sm://my-project//1",
        Error::MissingSecretName { location: "sm://my-project//1".into() }
    )]
    #[case::sm_resource_name_empty_secret(
        "sm://projects/my-project/secrets/",
        Error::MissingSecretName { location: "sm://projects/my-project/secrets/".into() }
    )]
    #[case::env_empty("env://", Error::MissingEnvVarName { location: "env://".into() })]
    #[case::env_blank("env://   ", Error::MissingEnvVarName { location: "env://".into() })]
    fn parse_rejects_invalid_location(#[case] location: &str, #[case] expected: Error) {
        assert_eq!(location.parse::<SecretLocation>(), Err(expected));
    }

    #[rstest]
    #[case::lowercase("api_key")]
    #[case::mixed_case("Api_Key2")]
    #[case::starts_with_underscore("_PRIVATE")]
    #[case::underscore_only("_")]
    #[case::single_letter("A")]
    fn parse_accepts_posix_env_var_name(#[case] name: &str) {
        assert_eq!(
            format!("env://{name}").parse::<SecretLocation>(),
            Ok(env(name))
        );
    }

    const ENV_CHARSET_REASON: &str = "must contain only ASCII letters, digits and `_`";
    const ENV_LEADING_DIGIT_REASON: &str = "must not start with a digit";

    #[rstest]
    #[case::starts_with_digit("1API_KEY", ENV_LEADING_DIGIT_REASON)]
    #[case::digits_only("123", ENV_LEADING_DIGIT_REASON)]
    #[case::with_equals("A=B", ENV_CHARSET_REASON)]
    #[case::with_nul("A\0B", ENV_CHARSET_REASON)]
    #[case::with_hyphen("API-KEY", ENV_CHARSET_REASON)]
    #[case::with_dot("api.key", ENV_CHARSET_REASON)]
    #[case::with_inner_space("API KEY", ENV_CHARSET_REASON)]
    #[case::non_ascii("ÄPI_KEY", ENV_CHARSET_REASON)]
    #[case::leading_hyphen_and_digit("-1", ENV_CHARSET_REASON)]
    fn parse_rejects_non_posix_env_var_name(#[case] name: &str, #[case] reason: &'static str) {
        let location = format!("env://{name}");
        assert_eq!(
            location.parse::<SecretLocation>(),
            Err(Error::InvalidEnvVarName {
                location: location.clone(),
                name: name.to_owned(),
                reason,
            })
        );
    }

    #[rstest]
    #[case::min_length("abcdef")]
    #[case::max_length(&format!("a{}", "b".repeat(29)))]
    #[case::with_digits_and_hyphens("my-project-123")]
    #[case::project_number("1")]
    fn parse_accepts_valid_project(#[case] project: &str) {
        assert_eq!(
            format!("sm://{project}/my-secret/1").parse::<SecretLocation>(),
            Ok(sm_in(project, "my-secret", "1"))
        );
    }

    #[rstest]
    #[case::uppercase("My-Project", "must contain only lowercase letters, digits and `-`")]
    #[case::underscore("my_project", "must contain only lowercase letters, digits and `-`")]
    #[case::domain_scoped(
        "example.com:my-project",
        "must contain only lowercase letters, digits and `-`"
    )]
    #[case::starts_with_digit("1-project", "must start with a lowercase letter")]
    #[case::starts_with_hyphen("-project", "must start with a lowercase letter")]
    #[case::ends_with_hyphen("my-project-", "must not end with `-`")]
    #[case::too_short("abcde", "must be 6 to 30 characters")]
    #[case::too_long(&format!("a{}", "b".repeat(30)), "must be 6 to 30 characters")]
    fn parse_rejects_invalid_project(#[case] project: &str, #[case] reason: &'static str) {
        let location = format!("sm://{project}/my-secret/1");
        assert_eq!(
            location.parse::<SecretLocation>(),
            Err(Error::InvalidProject {
                location: location.clone(),
                project: project.to_owned(),
                reason,
            })
        );
    }

    #[test]
    fn parse_validates_project_in_resource_name() {
        let location = "sm://projects/My-Project/secrets/my-secret";
        assert_eq!(
            location.parse::<SecretLocation>(),
            Err(Error::InvalidProject {
                location: location.to_owned(),
                project: "My-Project".to_owned(),
                reason: "must contain only lowercase letters, digits and `-`",
            })
        );
    }

    #[test]
    fn parse_rejects_too_long_secret_name() {
        let name = "a".repeat(MAX_SECRET_NAME_LEN + 1);
        let location = format!("sm://{name}");
        assert_eq!(
            location.parse::<SecretLocation>(),
            Err(Error::InvalidSecretName {
                location: location.clone(),
                name,
                reason: "must be at most 255 characters",
            })
        );
    }

    #[test]
    fn parse_accepts_max_length_secret_name() {
        let name = "a".repeat(MAX_SECRET_NAME_LEN);
        let location = format!("sm://{name}");
        assert_eq!(location.parse::<SecretLocation>(), Ok(sm(&name, "latest")));
    }

    #[rstest]
    #[case::none("none://", "none://")]
    #[case::sm_without_version("sm://my-secret", "sm://my-secret/latest")]
    #[case::sm_with_version("sm:// my-secret / 3 ", "sm://my-secret/3")]
    #[case::sm_with_project("sm://my-project/my-secret/3", "sm://my-project/my-secret/3")]
    #[case::sm_with_project_without_version(
        "sm://my-project/my-secret/",
        "sm://my-project/my-secret/latest"
    )]
    #[case::sm_resource_name(
        "sm://projects/my-project/secrets/my-secret",
        "sm://my-project/my-secret/latest"
    )]
    #[case::env("env:// API_KEY ", "env://API_KEY")]
    fn display_returns_normalized_location(#[case] location: &str, #[case] expected: &str) {
        let parsed: SecretLocation = location.parse().unwrap();
        assert_eq!(parsed.to_string(), expected);
        assert_eq!(expected.parse::<SecretLocation>(), Ok(parsed));
    }

    #[rstest]
    #[case::ok("sm://my-secret/2", Ok(item(None, "my-secret", "2")))]
    #[case::with_project(
        "sm://my-project/my-secret/2",
        Ok(item(Some("my-project"), "my-secret", "2"))
    )]
    #[case::other_scheme(
        "env://API_KEY",
        Err(Error::UnsupportedScheme {
            location: "env://API_KEY".into(),
            scheme: "env".into(),
            expected: "`sm://`",
        })
    )]
    fn secret_manager_item_parse(
        #[case] location: &str,
        #[case] expected: Result<SecretManagerItem>,
    ) {
        assert_eq!(location.parse::<SecretManagerItem>(), expected);
    }
}
