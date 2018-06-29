mod common {
    use std::collections::BTreeMap;

    pub type Map<K, V> = BTreeMap<K, V>;
}

mod meta {
    use super::common::Map;
    use std::fmt;

    type ValuePath = Vec<String>;

    pub type ValueError = String;
    // #[derive(Debug, PartialEq, Eq)]
    // pub struct ValueError {
    //     pub message: String,
    // }

    // impl From<String> for ValueError {
    //     fn from(message: String) -> Self {
    //         ValueError { message }
    //     }
    // }

    // impl<'a> From<&'a str> for ValueError {
    //     fn from(message: &'a str) -> Self {
    //         ValueError {
    //             message: message.into(),
    //         }
    //     }
    // }

    // #[derive(Debug, Deserialize)]
    // pub struct Annotation {
    //     pub rule: String,
    //     pub note: Option<String>,
    //     pub from: Option<u16>,
    //     pub to: Option<u16>,
    // }

    #[derive(Debug, Default, Deserialize, PartialEq)]
    pub struct ValueMeta {
        pub errors: Vec<ValueError>,
        // pub annotations: Vec<Annotation>,
        pub original_length: Option<u64>,
    }

    pub type EventMeta = Map<ValuePath, ValueMeta>;
}

mod schema {
    use super::meta::EventMeta;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    // TODO(ja): This is super strict now
    #[derive(Debug, Deserialize)]
    pub struct Values<T> {
        pub values: Vec<T>,
    }

    impl<T> Default for Values<T> {
        fn default() -> Self {
            Values { values: Vec::new() }
        }
    }

    impl<T> From<Vec<T>> for Values<T> {
        fn from(values: Vec<T>) -> Self {
            Values { values }
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(default)]
    pub struct Breadcrumb {
        pub timestamp: DateTime<Utc>,
        pub ty: String,
        pub category: Option<String>,
    }

    impl Default for Breadcrumb {
        fn default() -> Breadcrumb {
            Breadcrumb {
                timestamp: Utc::now(),
                ty: "default".into(),
                category: None,
            }
        }
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(default)]
    pub struct Event {
        pub id: Option<Uuid>,
        pub breadcrumbs: Values<Breadcrumb>,

        // TODO(ja): Fix this!
        #[serde(skip)]
        pub meta: EventMeta,
    }
}

mod annotated {
    use super::common::Map;
    use super::meta::ValueMeta;
    use super::schema::Values;
    use super::unexpected::UnexpectedType;
    use protocol::paths::Path;

    use std::rc::Rc;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer};
    use serde_json::{self, Value};
    use uuid::Uuid;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Maybe<T> {
        Valid(T),
        Invalid(UnexpectedType),
    }

    #[derive(Debug)]
    pub struct Annotated<T> {
        pub value: Option<T>,
        pub meta: ValueMeta,
        pub path: Option<String>,
    }

    impl<T> Annotated<T> {
        fn new(value: T) -> Self {
            Annotated {
                value: Some(value),
                meta: Default::default(),
                path: None,
            }
        }

        pub fn error(message: String) -> Self {
            Annotated {
                value: None,
                meta: ValueMeta {
                    errors: vec![message.into()],
                    ..Default::default()
                },
                path: None,
            }
        }
    }

    impl<'de, T> Deserialize<'de> for Annotated<T>
    where
        T: Deserialize<'de>
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let path = deserializer.get_state::<Rc<Path>>().map(|x| x.to_string());
            let mut annotated: Self = match Maybe::deserialize(deserializer)? {
                Maybe::Valid(value) => Annotated::new(value),
                Maybe::Invalid(u) => Annotated::error(format!("unexpected {}", u.0)),
            };
            annotated.path = path;
            Ok(annotated)
        }
    }
    impl<T> Default for Annotated<T> {
        fn default() -> Self {
            Annotated {
                value: None,
                meta: Default::default(),
                path: None,
            }
        }
    }

    #[derive(Debug, Default, Deserialize)]
    pub struct Breadcrumb {
        #[serde(default)]
        pub timestamp: Annotated<DateTime<Utc>>,
        #[serde(default)]
        pub ty: Annotated<String>,
        #[serde(default)]
        pub category: Annotated<Option<String>>,
    }

    #[derive(Debug, Default, Deserialize)]
    pub struct Event {
        #[serde(default, rename = "event_id")]
        pub id: Annotated<Option<Uuid>>,
        #[serde(default)]
        pub breadcrumbs: Annotated<Values<Annotated<Breadcrumb>>>,
    }

    pub type EventMeta = Map<String, ValueMeta>;

    #[derive(Deserialize)]
    pub struct EventMetaHelper {
        pub metadata: EventMeta,
    }
}

#[cfg(test)]
mod annotated_tests {
    use super::annotated::{Annotated, Event, EventMeta, EventMetaHelper};
    use super::meta::ValueMeta;
    use protocol::paths;
    use serde_json;

    #[test]
    fn test_annotated() {
        let json = r#"{
            "event_id": "864ee97977bf43ac96d74f7486d138ab",
            "breadcrumbs": {"values":[]}
        }"#;
        let jd = &mut serde_json::Deserializer::from_str(json);
        let e: Annotated<Event> = paths::deserialize(jd).unwrap();
        assert_eq!(e.value.unwrap().id.path, Some("event_id".to_string()));
    }

    #[test]
    fn test_foo() {
        let json = r#"{
            "event_id": "864ee97977bf43ac96d74f7486d138ab",
            "breadcrumbs": {"values":[{"timestamp": "2018-02-08T12:52:12Z"}]}
        }"#;
        let event = serde_json::from_str::<Annotated<Event>>(json).unwrap();
        assert!(event.meta.errors.is_empty());

        let json = r#"{
            "event_id": "864ee97977bf43ac96d74f7486d138ab",
            "breadcrumbs": {"values":[false]}
        }"#;
        let event = serde_json::from_str::<Annotated<Event>>(json).unwrap();
        assert_eq!(
            event.value.unwrap().breadcrumbs.value.unwrap().values[0]
                .meta
                .errors,
            vec!["unexpected boolean".to_string()]
        );

        let json = r#"{
            "event_id": "864ee97977bf43ac96d74f7486d138ab",
            "breadcrumbs": {"values":[false]},
            "metadata": {
                "breadcrumbs.values.0": {
                    "errors": ["original error"]
                }
            }
        }"#;

        /*
            "breadcrumbs": {
                "values": {
                    "0": {
                        "": {
                            "errors": ["original error"]
                        }
                    }
                }
            }
        */
        let event = serde_json::from_str::<Annotated<Event>>(json).unwrap();
        let meta = serde_json::from_str::<EventMetaHelper>(json)
            .unwrap()
            .metadata;
        assert_eq!(meta, {
            let mut m = EventMeta::new();
            m.insert(
                "breadcrumbs.values.0".into(),
                ValueMeta {
                    errors: vec!["original error".to_string()],
                    ..Default::default()
                },
            );
            m
        });
        // assert_eq!(
        //     event.value.unwrap().breadcrumbs.value.unwrap().values[0]
        //         .meta
        //         .errors,
        //     vec!["original error".into(), "unexpected boolean".into()]
        // );

        let json = r#"{
            "event_id": 42,
            "breadcrumbs": {"values":[{"timestamp": "2018-02-08T12:52:12Z"}]}
        }"#;
        let event = serde_json::from_str::<Annotated<Event>>(json).unwrap();
        println!("{:#?}", event);
        assert_eq!(
            event.value.unwrap().id.meta.errors,
            vec!["unexpected integer".to_string()]
        );

        // let json = r#"{
        //     "event_id": "864ee97977bf43ac96d74f7486d138ab",
        //     "breadcrumbs": {"values":[{"timestamp": "foo"}]}
        // }"#;
        // serde_json::from_str::<Annotated<Event>>(json).unwrap();

        // panic!();
    }
}

mod unexpected {
    use std::fmt;
    use std::ops;

    use serde::de::{Deserialize, Deserializer, Error, MapAccess, SeqAccess, Visitor};

    /// Deserializes into an unexpected type marker.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct UnexpectedType(pub &'static str);

    impl<'de> Deserialize<'de> for UnexpectedType {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<UnexpectedType, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(UnexpectedVisitor)
        }
    }

    impl ops::Deref for UnexpectedType {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

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

}

#[cfg(test)]
mod tests {}
