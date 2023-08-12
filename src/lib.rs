#![doc = include_str!("../README.md")]
use serde::{ser::SerializeMap, Serialize};

#[derive(Debug, Serialize)]
pub enum EntryDifference {
    ExtraKey,
    MissingKey,
    Value(Difference),
}

#[derive(Debug)]
pub struct DumbMap<K: Serialize, V: Serialize>(pub Vec<(K, V)>);

impl<K: Serialize, V: Serialize> Serialize for DumbMap<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

#[derive(Debug, Serialize)]
pub enum ArrayDifference {
    Length(usize, usize),
    Element(DumbMap<usize, Difference>),
}

#[derive(Debug, Serialize)]
pub enum Type {
    Null,
    Array,
    Bool,
    Object,
    String,
    Number,
}

#[derive(Debug, Serialize)]
pub enum ScalarDifference {
    Bool(bool, bool),
    String(String, String),
    Number(serde_json::Number, serde_json::Number),
}

#[derive(Debug, Serialize)]
pub struct TypeDifference {
    source_type: Type,
    target_value: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(tag = "difference_of")]
pub enum Difference {
    Scalar(ScalarDifference),
    Type(TypeDifference),
    Array(ArrayDifference),
    Object(DumbMap<String, EntryDifference>),
}

#[must_use]
pub fn arrays(a: Vec<serde_json::Value>, b: Vec<serde_json::Value>) -> Option<ArrayDifference> {
    let a_len = a.len();
    let b_len = b.len();

    if a_len != b_len {
        return Some(ArrayDifference::Length(a_len, b_len));
    }

    let element_differences = a
        .into_iter()
        .zip(b.into_iter())
        .enumerate()
        .filter_map(|(i, (a, b))| values(a, b).map(|diff| (i, diff)))
        .collect::<Vec<_>>();

    if element_differences.is_empty() {
        None
    } else {
        Some(ArrayDifference::Element(DumbMap(element_differences)))
    }
}

#[must_use]
pub fn objects(
    a: serde_json::Map<String, serde_json::Value>,
    mut b: serde_json::Map<String, serde_json::Value>,
) -> Option<DumbMap<String, EntryDifference>> {
    let mut value_differences = a
        .into_iter()
        .filter_map(|(key, a)| {
            let Some(b) = b.remove(&key) else {
                return Some((key, EntryDifference::MissingKey));
            };

            values(a, b).map(|diff| (key, EntryDifference::Value(diff)))
        })
        .collect::<Vec<_>>();

    value_differences.extend(
        b.into_iter()
            .map(|(extra_key, _)| (extra_key, EntryDifference::ExtraKey)),
    );

    if value_differences.is_empty() {
        None
    } else {
        Some(DumbMap(value_differences))
    }
}

impl From<serde_json::Value> for Type {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Type::Null,
            serde_json::Value::Bool(_) => Type::Bool,
            serde_json::Value::Number(_) => Type::Number,
            serde_json::Value::String(_) => Type::String,
            serde_json::Value::Array(_) => Type::Array,
            serde_json::Value::Object(_) => Type::Object,
        }
    }
}

#[must_use]
pub fn values(a: serde_json::Value, b: serde_json::Value) -> Option<Difference> {
    use serde_json::Value::{Array, Bool, Null, Number, Object, String};

    match (a, b) {
        (Null, Null) => None,
        (Bool(a), Bool(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::Bool(a, b)))
            }
        }
        (Number(a), Number(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::Number(a, b)))
            }
        }
        (String(a), String(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::String(a, b)))
            }
        }
        (Array(a), Array(b)) => arrays(a, b).map(Difference::Array),
        (Object(a), Object(b)) => objects(a, b).map(Difference::Object),
        (a, b) => Some(Difference::Type(TypeDifference {
            source_type: a.into(),
            target_value: b,
        })),
    }
}
