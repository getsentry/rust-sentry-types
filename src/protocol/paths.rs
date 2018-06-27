use serde::de::{self, Deserialize, DeserializeSeed, Visitor};
use std::fmt::{self, Display};

pub trait Process<C> {
    fn process(&mut self, path: &Path, ctx: &mut C);
}

/// Entry point. See crate documentation for an example.
pub fn deserialize<'de, D, F, T, C>(deserializer: D, ctx: &mut C) -> Result<T, D::Error>
where
    D: de::Deserializer<'de>,
    T: Deserialize<'de> + Process<C>,
{
    T::deserialize(Deserializer::new(deserializer, ctx))
}

/// Deserializer adapter that invokes a ctx with the path to every unused
/// field of the input.
pub struct Deserializer<'a, 'b, D, C: 'b> {
    de: D,
    ctx: &'b mut C,
    path: Path<'a>,
}

impl<'a, 'b, D, C: 'b> Deserializer<'a, 'b, D, C> {
    // The structs in this crate all hold their closure by &mut F. If they were
    // to contain F by value, any method taking &mut self (for example
    // SeqAccess::next_element_seed) would be forced to recurse with &mut
    // self.ctx, even if F is instantiated with a &mut already. This way
    // they contain &mut F and the &mut self methods can recurse with
    // self.ctx unchanged. This avoids blowing the recursion limit in
    // Cargo's use of this crate.
    //
    // https://github.com/dtolnay/serde-ignored/pull/1
    pub fn new(de: D, ctx: &'b mut C) -> Self {
        Deserializer {
            de: de,
            ctx,
            path: Path::Root,
        }
    }
}

/// Path to the current value in the input, like `dependencies.serde.typo1`.
pub enum Path<'a> {
    Root,
    Seq { parent: &'a Path<'a>, index: usize },
    Map { parent: &'a Path<'a>, key: String },
    Some { parent: &'a Path<'a> },
    NewtypeStruct { parent: &'a Path<'a> },
    NewtypeVariant { parent: &'a Path<'a> },
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        struct Parent<'a>(&'a Path<'a>);

        impl<'a> Display for Parent<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                match *self.0 {
                    Path::Root => Ok(()),
                    ref path => write!(formatter, "{}.", path),
                }
            }
        }

        match *self {
            Path::Root => formatter.write_str("."),
            Path::Seq { parent, index } => write!(formatter, "{}{}", Parent(parent), index),
            Path::Map { parent, ref key } => write!(formatter, "{}{}", Parent(parent), key),
            Path::Some { parent }
            | Path::NewtypeStruct { parent }
            | Path::NewtypeVariant { parent } => write!(formatter, "{}?", Parent(parent)),
        }
    }
}

/// Plain old forwarding impl except for `deserialize_ignored_any` which invokes
/// the ctx.
impl<'a, 'b, 'de, D, C: 'b> de::Deserializer<'de> for Deserializer<'a, 'b, D, C>
where
    D: de::Deserializer<'de>,
{
    type Error = D::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_any(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_bool(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_u8(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_u16(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_u32(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_u64(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_i8(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_i16(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_i32(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_i64(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_f32(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_f64(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_char(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_str(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_string(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_bytes(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_byte_buf(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_option(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_unit(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_unit_struct(name, Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_newtype_struct(name, Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_seq(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_tuple(len, Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_tuple_struct(name, len, Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_map(Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_struct(name, fields, Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_enum(name, variants, Wrap::new(visitor, self.ctx, &self.path))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        // (self.ctx)(self.path);

        self.de.deserialize_ignored_any(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, D::Error>
    where
        V: Visitor<'de>,
    {
        self.de
            .deserialize_identifier(Wrap::new(visitor, self.ctx, &self.path))
    }
}

/// Wrapper that attaches context to a `Visitor`, `SeqAccess`, `EnumAccess` or
/// `VariantAccess`.
struct Wrap<'a, 'b, X, C: 'b> {
    delegate: X,
    ctx: &'b mut C,
    path: &'a Path<'a>,
}

impl<'a, 'b, X, C> Wrap<'a, 'b, X, C> {
    fn new(delegate: X, ctx: &'b mut C, path: &'a Path<'a>) -> Self {
        Wrap {
            delegate: delegate,
            ctx: ctx,
            path: path,
        }
    }
}

/// Forwarding impl to preserve context.
impl<'a, 'b, 'de, X, C> Visitor<'de> for Wrap<'a, 'b, X, C>
where
    X: Visitor<'de>,
    C: 'b,
{
    type Value = X::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.delegate.expecting(formatter)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_bool(v)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i8(v)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i16(v)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i32(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i64(v)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u8(v)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u16(v)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u32(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u64(v)
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_f32(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_f64(v)
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_char(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_str(v)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_borrowed_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_string(v)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_unit()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.delegate.visit_some(Deserializer {
            de: deserializer,
            ctx: self.ctx,
            path: Path::Some { parent: self.path },
        })
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.delegate.visit_newtype_struct(Deserializer {
            de: deserializer,
            ctx: self.ctx,
            path: Path::NewtypeStruct { parent: self.path },
        })
    }

    fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        self.delegate
            .visit_seq(SeqAccess::new(visitor, self.ctx, self.path))
    }

    fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        self.delegate
            .visit_map(MapAccess::new(visitor, self.ctx, self.path))
    }

    fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::EnumAccess<'de>,
    {
        self.delegate
            .visit_enum(Wrap::new(visitor, self.ctx, self.path))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_bytes(v)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_byte_buf(v)
    }
}

/// Forwarding impl to preserve context.
impl<'a, 'b, 'de, X: 'a, C: 'b> de::EnumAccess<'de> for Wrap<'a, 'b, X, C>
where
    X: de::EnumAccess<'de>,
{
    type Error = X::Error;
    type Variant = Wrap<'a, 'b, X::Variant, C>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), X::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let ctx = self.ctx;
        let path = self.path;
        self.delegate
            .variant_seed(seed)
            .map(move |(v, vis)| (v, Wrap::new(vis, ctx, path)))
    }
}

/// Forwarding impl to preserve context.
impl<'a, 'b, 'de, X, C: 'b> de::VariantAccess<'de> for Wrap<'a, 'b, X, C>
where
    X: de::VariantAccess<'de>,
{
    type Error = X::Error;

    fn unit_variant(self) -> Result<(), X::Error> {
        self.delegate.unit_variant()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, X::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let path = Path::NewtypeVariant { parent: self.path };
        self.delegate
            .newtype_variant_seed(TrackedSeed::new(seed, self.ctx, path))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .tuple_variant(len, Wrap::new(visitor, self.ctx, self.path))
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .struct_variant(fields, Wrap::new(visitor, self.ctx, self.path))
    }
}

/// Seed that saves the string into the given optional during `visit_str` and
/// `visit_string`.
struct CaptureKey<'a, X> {
    delegate: X,
    key: &'a mut Option<String>,
}

impl<'a, X> CaptureKey<'a, X> {
    fn new(delegate: X, key: &'a mut Option<String>) -> Self {
        CaptureKey {
            delegate: delegate,
            key: key,
        }
    }
}

/// Forwarding impl.
impl<'a, 'de, X> DeserializeSeed<'de> for CaptureKey<'a, X>
where
    X: DeserializeSeed<'de>,
{
    type Value = X::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<X::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.delegate
            .deserialize(CaptureKey::new(deserializer, self.key))
    }
}

/// Forwarding impl.
impl<'a, 'de, X> de::Deserializer<'de> for CaptureKey<'a, X>
where
    X: de::Deserializer<'de>,
{
    type Error = X::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_any(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_bool(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_u8(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_u16(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_u32(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_u64(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_i8(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_i16(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_i32(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_i64(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_f32(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_f64(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_char(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_str(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_string(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_bytes(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_byte_buf(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_option(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_unit(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_unit_struct(name, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_newtype_struct(name, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_seq(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_tuple(len, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_tuple_struct(name, len, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_map(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_struct(name, fields, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_enum(name, variants, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_ignored_any(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, X::Error>
    where
        V: Visitor<'de>,
    {
        self.delegate
            .deserialize_identifier(CaptureKey::new(visitor, self.key))
    }
}

/// Forwarding impl except `visit_str` and `visit_string` which save the string.
impl<'a, 'de, X> Visitor<'de> for CaptureKey<'a, X>
where
    X: Visitor<'de>,
{
    type Value = X::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.delegate.expecting(formatter)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_bool(v)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i8(v)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i16(v)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i32(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i64(v)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u8(v)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u16(v)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u32(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u64(v)
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_f32(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_f64(v)
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_char(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        *self.key = Some(v.to_owned());
        self.delegate.visit_str(v)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        *self.key = Some(v.to_owned());
        self.delegate.visit_borrowed_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        *self.key = Some(v.clone());
        self.delegate.visit_string(v)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_unit()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.delegate.visit_some(deserializer)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.delegate.visit_newtype_struct(deserializer)
    }

    fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        self.delegate.visit_seq(visitor)
    }

    fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        self.delegate.visit_map(visitor)
    }

    fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::EnumAccess<'de>,
    {
        self.delegate.visit_enum(visitor)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_bytes(v)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_byte_buf(v)
    }
}

/// Seed used for map values, sequence elements and newtype variants to track
/// their path.
struct TrackedSeed<'a, X, C: 'a> {
    seed: X,
    ctx: &'a mut C,
    path: Path<'a>,
}

impl<'a, X, C> TrackedSeed<'a, X, C> {
    fn new(seed: X, ctx: &'a mut C, path: Path<'a>) -> Self {
        TrackedSeed {
            seed: seed,
            ctx: ctx,
            path: path,
        }
    }
}

impl<'a, 'de, X, C> DeserializeSeed<'de> for TrackedSeed<'a, X, C>
where
    X: DeserializeSeed<'de>,
{
    type Value = X::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<X::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.seed.deserialize(Deserializer {
            de: deserializer,
            ctx: self.ctx,
            path: self.path,
        })
    }
}

/// Seq visitor that tracks the index of its elements.
struct SeqAccess<'a, 'b, X, C: 'b> {
    delegate: X,
    ctx: &'b mut C,
    path: &'a Path<'a>,
    index: usize,
}

impl<'a, 'b, X, C> SeqAccess<'a, 'b, X, C> {
    fn new(delegate: X, ctx: &'b mut C, path: &'a Path<'a>) -> Self {
        SeqAccess {
            delegate: delegate,
            ctx: ctx,
            path: path,
            index: 0,
        }
    }
}

/// Forwarding impl to preserve context.
impl<'a, 'b, 'de, X, C: 'b> de::SeqAccess<'de> for SeqAccess<'a, 'b, X, C>
where
    X: de::SeqAccess<'de>,
{
    type Error = X::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, X::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let path = Path::Seq {
            parent: self.path,
            index: self.index,
        };
        self.index += 1;
        self.delegate
            .next_element_seed(TrackedSeed::new(seed, self.ctx, path))
    }

    fn size_hint(&self) -> Option<usize> {
        self.delegate.size_hint()
    }
}

/// Map visitor that captures the string value of its keys and uses that to
/// track the path to its values.
struct MapAccess<'a, 'b, X, C: 'b> {
    delegate: X,
    ctx: &'b mut C,
    path: &'a Path<'a>,
    key: Option<String>,
}

impl<'a, 'b, X, C: 'b> MapAccess<'a, 'b, X, C> {
    fn new(delegate: X, ctx: &'b mut C, path: &'a Path<'a>) -> Self {
        MapAccess {
            delegate: delegate,
            ctx: ctx,
            path: path,
            key: None,
        }
    }

    fn key<E>(&mut self) -> Result<String, E>
    where
        E: de::Error,
    {
        self.key.take().ok_or_else(|| E::custom("non-string key"))
    }
}

impl<'a, 'b, 'de, X, C: 'b> de::MapAccess<'de> for MapAccess<'a, 'b, X, C>
where
    X: de::MapAccess<'de>,
{
    type Error = X::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, X::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.delegate
            .next_key_seed(CaptureKey::new(seed, &mut self.key))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, X::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let path = Path::Map {
            parent: self.path,
            key: self.key()?,
        };
        self.delegate
            .next_value_seed(TrackedSeed::new(seed, self.ctx, path))
    }

    fn size_hint(&self) -> Option<usize> {
        self.delegate.size_hint()
    }
}
