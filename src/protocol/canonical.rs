use uuid::Uuid;

use serde::de::{Deserializer, Deserialize};

use protocol::unexpected::UnexpectedType;

#[derive(Debug)]
pub struct Annotation {
    pub rule: String,
    pub note: Option<String>,
    pub from: Option<u16>,
    pub to: Option<u16>,
}

#[derive(Debug, Default)]
pub struct ValueMeta {
    pub errors: Vec<String>,
    pub annotations: Vec<Annotation>,
    pub original_length: Option<u64>,
}

#[derive(Default, Debug)]
pub struct Annotated<T> {
    pub value: T,
    pub meta: ValueMeta,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum LenientValueHelper<T> {
    Valid(T),
    UnexpectedType(UnexpectedType),
}

impl<'de, T> Deserialize<'de> for Annotated<T>
where
    T: Deserialize<'de> + Default,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value: LenientValueHelper<T> = Deserialize::deserialize(deserializer)?;
        Ok(match value {
            LenientValueHelper::Valid(value) => {
                Annotated {
                    value: value,
                    meta: Default::default()
                }
            }
            LenientValueHelper::UnexpectedType(UnexpectedType(ty)) => {
                Annotated {
                    value: Default::default(),
                    meta: ValueMeta {
                        errors: vec![format!("unexpected type {}", ty)],
                        ..Default::default()
                    }
                }
            }
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct Event {
    #[serde(rename = "event_id")]
    pub id: Annotated<Option<Uuid>>,
}

#[test]
fn test_event() {
    use serde_json;
    let evt: Event = serde_json::from_str(r#"{"event_id": 42}"#).unwrap();
    println!("{:#?}", &evt);
    let evt: Event = serde_json::from_str(r#"{"event_id": "50e8353c-4d8d-42e1-b5fb-058860c6317c"}"#).unwrap();
    println!("{:#?}", &evt);
    panic!();
}

/*
impl<'de, T> Deserialize<'de> for (T, MetaData)
where
    T: Deserialize<'de> + Default,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let annotated_de = AnnotatedDeserializer::new(deserializer);
        let res = T::deserialize(annotated_de)?;
        Ok(res, annotated_de.take_meta())
    }
}

#[test]
fn test_event_ideal() {
    use serde_json;
    let jd = &mut serde_json::Deserializer::from_str(r#"{"event_id": 42}"#);
    let (event: Event, meta: MetaData) = deserialize_with_metadata(jd);
}
*/
