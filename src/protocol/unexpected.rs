use std::fmt;

use serde::de::{Deserialize, Deserializer, Error, MapAccess, SeqAccess, Visitor};

/// Deserializes into an unexpected type marker.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct UnexpectedType(pub &'static str);

struct UnexpectedVisitor;

impl<'de> Visitor<'de> for UnexpectedVisitor {
    type Value = UnexpectedType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("anything at all")
    }

    #[inline]
    fn visit_bool<E>(self, x: bool) -> Result<Self::Value, E> {
        let _ = x;
        Ok(UnexpectedType("boolean"))
    }

    #[inline]
    fn visit_i64<E>(self, x: i64) -> Result<Self::Value, E> {
        let _ = x;
        Ok(UnexpectedType("integer"))
    }

    #[inline]
    fn visit_u64<E>(self, x: u64) -> Result<Self::Value, E> {
        let _ = x;
        Ok(UnexpectedType("integer"))
    }

    #[inline]
    fn visit_f64<E>(self, x: f64) -> Result<Self::Value, E> {
        let _ = x;
        Ok(UnexpectedType("integer"))
    }

    #[inline]
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let _ = s;
        Ok(UnexpectedType("string"))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UnexpectedType("null"))
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UnexpectedType::deserialize(deserializer)
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UnexpectedType::deserialize(deserializer)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UnexpectedType("null"))
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(UnexpectedType(..)) = seq.next_element()? {
            // Gobble
        }
        Ok(UnexpectedType("array"))
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some((UnexpectedType(..), UnexpectedType(..))) = map.next_entry()? {
            // Gobble
        }
        Ok(UnexpectedType("object"))
    }

    #[inline]
    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let _ = bytes;
        Ok(UnexpectedType("bytes"))
    }
}

impl<'de> Deserialize<'de> for UnexpectedType {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<UnexpectedType, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UnexpectedVisitor)
    }
}

#[test]
fn test_unexpected_type() {
    use serde_json;
    let data: UnexpectedType = serde_json::from_str("[\"foobarbaz\"]").unwrap();
    assert_eq!(data, UnexpectedType("array"));

    let data: UnexpectedType = serde_json::from_str("42").unwrap();
    assert_eq!(data, UnexpectedType("integer"));

    let data: UnexpectedType = serde_json::from_str("\"test\"").unwrap();
    assert_eq!(data, UnexpectedType("string"));
}
