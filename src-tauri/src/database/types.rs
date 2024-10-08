use base64::prelude::*;
use base64::DecodeError;
use derive_more::derive::Deref;
use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
    Database, Decode, Encode, Sqlite,
};

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, specta::Type)]
pub struct SqliteBool(bool);

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

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type, Deref, PartialEq)]
pub struct Base64(String);

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

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct NullableBase64(Option<Base64>);

impl From<Option<Base64>> for NullableBase64 {
    fn from(value: Option<Base64>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
impl NullableBase64 {
    pub fn inner(&self) -> Option<&Base64> {
        self.0.as_ref()
    }

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
