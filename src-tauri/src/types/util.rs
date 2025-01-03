use anyhow::anyhow;
use base64::prelude::*;
use derive_more::derive::Deref;
use serde::de;
use serde::de::Visitor;
use serde::Deserializer;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{
    sqlite::{SqliteTypeInfo, SqliteValueRef},
    Database, Decode, Encode, Sqlite,
};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, specta::Type)]
#[serde(transparent)]
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

#[derive(Debug, Clone, Deref, FromRow, specta::Type)]
pub struct BytesBase64URL(#[specta(type = String)] Vec<u8>);

impl From<Vec<u8>> for BytesBase64URL {
    fn from(value: Vec<u8>) -> Self {
        BytesBase64URL(value)
    }
}

impl Serialize for BytesBase64URL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&BASE64_URL_SAFE_NO_PAD.encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for BytesBase64URL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UUIDv7Visitor;

        impl Visitor<'_> for UUIDv7Visitor {
            type Value = BytesBase64URL;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a base64 encoded string")
            }

            fn visit_str<E>(self, value: &str) -> Result<BytesBase64URL, E>
            where
                E: de::Error,
            {
                let decoded = BASE64_URL_SAFE_NO_PAD.decode(value).map_err(|e| {
                    E::custom(format!("failed to decode uuid from base64 string: {}", e))
                })?;
                Ok(BytesBase64URL(decoded))
            }
        }

        deserializer.deserialize_str(UUIDv7Visitor)
    }
}

impl sqlx::Type<Sqlite> for BytesBase64URL {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for BytesBase64URL {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;

        Ok(BytesBase64URL(bytes.to_vec()))
    }
}

impl<'r> Encode<'r, Sqlite> for BytesBase64URL {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let bytes = self.0.to_vec();
        <&Vec<u8> as Encode<Sqlite>>::encode(&bytes, buf)
    }
}

#[derive(Debug, Clone, Copy, Deref, PartialEq, Eq, Hash, FromRow, specta::Type)]
pub struct UUIDv7Base64URL(#[specta(type = String)] [u8; 16]);

impl UUIDv7Base64URL {
    pub fn new() -> Self {
        UUIDv7Base64URL(uuidv7::create_raw())
    }
}

impl Default for UUIDv7Base64URL {
    fn default() -> Self {
        Self::new()
    }
}

impl From<UUIDv7Base64URL> for String {
    fn from(value: UUIDv7Base64URL) -> Self {
        BASE64_URL_SAFE_NO_PAD.encode(value.0)
    }
}

impl fmt::Display for UUIDv7Base64URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE64_URL_SAFE_NO_PAD.encode(self.0))
    }
}

impl TryFrom<String> for UUIDv7Base64URL {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(value)?;

        if decoded.len() != 16 {
            return Err(anyhow!("invalid uuid length"));
        }

        let mut result_slice = [0u8; 16];
        result_slice.copy_from_slice(&decoded);

        Ok(UUIDv7Base64URL(result_slice))
    }
}

impl TryFrom<&str> for UUIDv7Base64URL {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(value)?;

        if decoded.len() != 16 {
            return Err(anyhow!("invalid uuid length"));
        }

        let mut result_slice = [0u8; 16];
        result_slice.copy_from_slice(&decoded);

        Ok(UUIDv7Base64URL(result_slice))
    }
}

impl TryFrom<Vec<u8>> for UUIDv7Base64URL {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != 16 {
            return Err(anyhow!("invalid uuid length"));
        }

        let mut result_slice = [0u8; 16];
        result_slice.copy_from_slice(&value);

        Ok(UUIDv7Base64URL(result_slice))
    }
}

impl Serialize for UUIDv7Base64URL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&BASE64_URL_SAFE_NO_PAD.encode(self.0))
    }
}

impl<'de> Deserialize<'de> for UUIDv7Base64URL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UUIDv7Visitor;

        impl Visitor<'_> for UUIDv7Visitor {
            type Value = UUIDv7Base64URL;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a base64 encoded string")
            }

            fn visit_str<E>(self, value: &str) -> Result<UUIDv7Base64URL, E>
            where
                E: de::Error,
            {
                let decoded = BASE64_URL_SAFE_NO_PAD.decode(value).map_err(|e| {
                    E::custom(format!("failed to decode uuid from base64 string: {}", e))
                })?;

                if decoded.len() != 16 {
                    return Err(E::custom("invalid uuid length"));
                }

                let mut result_slice = [0u8; 16];
                result_slice.copy_from_slice(&decoded);

                Ok(UUIDv7Base64URL(result_slice))
            }
        }

        deserializer.deserialize_str(UUIDv7Visitor)
    }
}

impl sqlx::Type<Sqlite> for UUIDv7Base64URL {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'r> Decode<'r, Sqlite> for UUIDv7Base64URL {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;

        if bytes.len() != 16 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid UUID length",
            )));
        }

        let mut result_slice = [0u8; 16];
        result_slice.copy_from_slice(bytes);

        Ok(UUIDv7Base64URL(result_slice))
    }
}

impl<'r> Encode<'r, Sqlite> for UUIDv7Base64URL {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let bytes = self.0.to_vec();
        <&Vec<u8> as Encode<Sqlite>>::encode(&bytes, buf)
    }
}
