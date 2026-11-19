//! `serde`によるデシリアライズの共通処理

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Deserializer, Visitor};

use crate::errors::Error;

/// 文字列を受け取り、[`FromStr`]で解析してデシリアライズする。
///
/// `expecting`は、文字列以外の値を受け取ったときのエラーメッセージに使う。
pub(crate) fn deserialize_from_str<'de, D, T>(
    deserializer: D,
    expecting: &'static str,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr<Err = Error>,
{
    deserializer.deserialize_str(FromStrVisitor {
        expecting,
        marker: PhantomData,
    })
}

/// 文字列を[`FromStr`]で解析する[`Visitor`]
struct FromStrVisitor<T> {
    /// 期待する値の説明
    expecting: &'static str,

    marker: PhantomData<T>,
}

impl<T> Visitor<'_> for FromStrVisitor<T>
where
    T: FromStr<Err = Error>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.expecting)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
        v.parse().map_err(E::custom)
    }
}
