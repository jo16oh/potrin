use base64::prelude::*;
use derive_more::derive::Deref;
use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
    Database, Decode, Encode, Sqlite,
};

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Local,
    Remote,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type, Deref)]
pub struct Base64String(String);

impl Base64String {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Base64String(BASE64_STANDARD.encode(bytes))
    }

    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let string = self.as_bytes();
        let mut buffer = Vec::<u8>::new();
        BASE64_STANDARD.decode_vec(string, &mut buffer)?;
        Ok(buffer)
    }
}

impl sqlx::Type<Sqlite> for Base64String {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for Base64String {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;
        let base64 = BASE64_STANDARD.encode(bytes);
        Ok(Base64String(base64))
    }
}

impl<'r> Encode<'r, Sqlite> for Base64String {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self.to_bytes() {
            Ok(bytes) => <&Vec<u8> as Encode<Sqlite>>::encode(&bytes, buf),
            Err(_) => Ok(IsNull::Yes),
        }
    }
}

impl From<Vec<u8>> for Base64String {
    fn from(value: Vec<u8>) -> Self {
        Base64String(BASE64_STANDARD.encode(value))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type, Deref)]
pub struct NullableBase64String(pub Option<Base64String>);

impl NullableBase64String {
    pub fn none() -> Self {
        NullableBase64String(None)
    }
}

impl sqlx::Type<Sqlite> for NullableBase64String {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for NullableBase64String {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let base64string =
            <Option<Vec<u8>> as Decode<Sqlite>>::decode(value)?.map(Base64String::from_bytes);
        Ok(NullableBase64String(base64string))
    }
}

impl<'r> Encode<'r, Sqlite> for NullableBase64String {
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

impl From<Option<Vec<u8>>> for NullableBase64String {
    fn from(opt: Option<Vec<u8>>) -> Self {
        let base64 = opt.map(Base64String::from_bytes);
        NullableBase64String(base64)
    }
}

impl From<Base64String> for NullableBase64String {
    fn from(base64: Base64String) -> Self {
        NullableBase64String(Some(base64))
    }
}
