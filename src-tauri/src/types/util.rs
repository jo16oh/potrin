use base64::prelude::*;
use base64::DecodeError;
use derive_more::derive::Deref;
use serde::de;
use serde::de::Visitor;
use serde::Deserializer;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
    Database, Decode, Encode, Sqlite,
};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    Local,
    Remote,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, specta::Type)]
pub struct SqliteBool(bool);

impl Serialize for SqliteBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(self.0)
    }
}

impl<'de> Deserialize<'de> for SqliteBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SqliteBoolVisitor;

        impl<'de> Visitor<'de> for SqliteBoolVisitor {
            type Value = SqliteBool;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a boolean value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<SqliteBool, E>
            where
                E: de::Error,
            {
                Ok(SqliteBool(value))
            }
        }

        deserializer.deserialize_bool(SqliteBoolVisitor)
    }
}

impl sqlx::Type<Sqlite> for SqliteBool {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <&i64 as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for SqliteBool {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <i64 as Decode<Sqlite>>::decode(value)?;
        Ok(Self(value != 0))
    }
}

impl<'r> Encode<'r, Sqlite> for SqliteBool {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as Encode<Sqlite>>::encode(if self.0 { 1 } else { 0 }, buf)
    }
}

impl From<i64> for SqliteBool {
    fn from(value: i64) -> Self {
        Self(value != 0)
    }
}

// impl SqliteBool {
//     pub fn new(value: bool) -> Self {
//         Self(value)
//     }
// }

#[derive(Clone, Debug, specta::Type, Deref, PartialEq, Eq, Hash)]
pub struct Base64(String);

impl Serialize for Base64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Base64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Base64Visitor;

        impl<'de> Visitor<'de> for Base64Visitor {
            type Value = Base64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a base64 encoded string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Base64, E>
            where
                E: de::Error,
            {
                Ok(Base64(value.to_string()))
            }
        }

        deserializer.deserialize_str(Base64Visitor)
    }
}

impl Base64 {
    pub fn to_bytes(&self) -> Result<Vec<u8>, DecodeError> {
        let string = self.as_bytes();
        let mut buffer = Vec::<u8>::new();
        BASE64_STANDARD.decode_vec(string, &mut buffer)?;
        Ok(buffer)
    }
}

impl sqlx::Type<Sqlite> for Base64 {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for Base64 {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;
        let base64 = BASE64_STANDARD.encode(bytes);
        Ok(Base64(base64))
    }
}

impl<'r> Encode<'r, Sqlite> for Base64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&Vec<u8> as Encode<Sqlite>>::encode(&self.to_bytes()?, buf)
    }
}

impl From<Vec<u8>> for Base64 {
    fn from(value: Vec<u8>) -> Self {
        Base64(BASE64_STANDARD.encode(value))
    }
}

impl From<String> for Base64 {
    fn from(value: String) -> Self {
        Base64(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct NullableBase64(Option<Base64>);

impl From<Option<Base64>> for NullableBase64 {
    fn from(value: Option<Base64>) -> Self {
        Self(value)
    }
}

impl NullableBase64 {
    #[cfg(test)]
    pub fn as_ref(&self) -> Option<&Base64> {
        self.0.as_ref()
    }

    pub fn into_option(self) -> Option<Base64> {
        self.0
    }

    #[cfg(test)]
    pub fn none() -> Self {
        Self(None)
    }
}

impl sqlx::Type<Sqlite> for NullableBase64 {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for NullableBase64 {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let base64string = <Option<Vec<u8>> as Decode<Sqlite>>::decode(value)?.map(Base64::from);
        Ok(NullableBase64(base64string))
    }
}

impl<'r> Encode<'r, Sqlite> for NullableBase64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match &self.0 {
            Some(base64_string) => base64_string.encode_by_ref(buf),
            None => Ok(IsNull::Yes),
        }
    }
}

impl From<Option<Vec<u8>>> for NullableBase64 {
    fn from(opt: Option<Vec<u8>>) -> Self {
        let base64 = opt.map(Base64::from);
        NullableBase64(base64)
    }
}

impl From<Base64> for NullableBase64 {
    fn from(base64: Base64) -> Self {
        NullableBase64(Some(base64))
    }
}

impl From<Vec<u8>> for NullableBase64 {
    fn from(value: Vec<u8>) -> Self {
        NullableBase64::from(Base64::from(value))
    }
}
