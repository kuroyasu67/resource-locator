//! `<scheme>://<rest>`形式の文字列の分解

use std::path::PathBuf;

use crate::errors::{Error, Result};

/// スキームの区切り文字
pub(crate) const SEPARATOR: &str = "://";

/// ローカルファイルのスキーム
pub(crate) const LOCAL_SCHEME: &str = "local";

/// 前後の空白を除いたロケーション
#[derive(Debug, Clone, Copy)]
pub(crate) struct Parts<'a> {
    /// 前後の空白を除いたロケーション全体
    pub location: &'a str,

    /// スキーム(`://`を含まない)
    pub scheme: &'a str,

    /// `://`より後ろの文字列
    pub rest: &'a str,
}

/// 文字列をスキームとそれ以降に分解する。
///
/// # Errors
///
/// - 空、または空白のみの場合は[`Error::Empty`]
/// - `://`を含まない場合は[`Error::MissingScheme`]
pub(crate) fn split(s: &str) -> Result<Parts<'_>> {
    let location = s.trim();
    if location.is_empty() {
        return Err(Error::Empty);
    }
    let (scheme, rest) = location
        .split_once(SEPARATOR)
        .ok_or_else(|| Error::MissingScheme {
            location: location.to_owned(),
        })?;
    Ok(Parts {
        location,
        scheme,
        rest,
    })
}

impl Parts<'_> {
    /// スキームが対応していないことを示すエラーを返す。
    pub(crate) fn unsupported(&self, expected: &'static str) -> Error {
        Error::UnsupportedScheme {
            location: self.location.to_owned(),
            scheme: self.scheme.to_owned(),
            expected,
        }
    }

    /// `local://`より後ろの文字列を、前後の空白を除いてローカルファイルのパスとして返す。
    ///
    /// # Errors
    ///
    /// パスが空、または空白のみの場合は[`Error::MissingLocalPath`]
    pub(crate) fn local_path(&self) -> Result<PathBuf> {
        let path = self.rest.trim();
        if path.is_empty() {
            return Err(Error::MissingLocalPath {
                location: self.location.to_owned(),
            });
        }
        Ok(PathBuf::from(path))
    }
}
