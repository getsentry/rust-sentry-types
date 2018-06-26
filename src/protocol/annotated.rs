use std::collections::BTreeMap;
use std::rc::Rc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
struct Values<T> {
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

type FieldPath = Vec<String>;

#[derive(Debug)]
struct FieldError {
    pub message: String,
}

impl From<String> for FieldError {
    fn from(message: String) -> Self {
        FieldError { message }
    }
}

#[derive(Debug)]
struct Annotation {
    pub rule: String,
    pub note: Option<String>,
    pub from: Option<u16>,
    pub to: Option<u16>,
}

#[derive(Debug, Default)]
struct FieldMeta {
    pub errors: Vec<FieldError>,
    pub annotations: Vec<Annotation>,
    pub original_length: Option<u64>,
}

mod annotated {
    use super::*;

    use serde::de::{Deserialize, Deserializer};
    use serde_json::Value;

    #[derive(Default)]
    struct Annotated<T> {
        pub value: T,
        pub meta: FieldMeta,
        submeta: BTreeMap<FieldPath, Rc<FieldMeta>>,
    }

    impl<T> From<T> for Annotated<T> {
        fn from(value: T) -> Self {
            Annotated {
                value,
                meta: Default::default(),
                submeta: Default::default(),
            }
        }
    }

    impl<T> Annotated<T> {
        fn error(message: String, default: T) -> Self {
            Annotated {
                value: default,
                meta: FieldMeta {
                    errors: vec![message.into()],
                    ..Default::default()
                },
                submeta: Default::default(),
            }
        }
    }

    struct Breadcrumb {
        pub timestamp: Annotated<DateTime<Utc>>,
        pub ty: Annotated<String>,
        pub category: Annotated<Option<String>>,
    }

    #[derive(Default)]
    struct Event {
        pub id: Annotated<Option<Uuid>>,
        pub breadcrumbs: Annotated<Values<Annotated<Breadcrumb>>>,
    }

    impl From<Value> for Annotated<Event> {
        fn from(value: Value) -> Annotated<Event> {
            let map = match value {
                Value::Object(map) => map,
                _ => return Annotated::error("".into(), Default::default()),
            };
            unimplemented!();
        }
    }

    impl<'de> Deserialize<'de> for Annotated<Event> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Value::deserialize(deserializer)?.into()
        }
    }
}

mod canonical {
    use super::*;

    struct Breadcrumb {
        pub timestamp: DateTime<Utc>,
        pub ty: String,
        pub category: Option<String>,
    }

    struct Event {
        pub id: Option<Uuid>,
        pub breadcrumbs: Values<Breadcrumb>,
        pub meta: BTreeMap<FieldPath, FieldMeta>,
    }

    impl From<annotated::Annotated<annotated::Event>> for Event {}
}
