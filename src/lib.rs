use serde::{ser::SerializeMap, Serialize};

#[derive(Debug, Serialize)]
pub enum EntryDifference {
    ExtraKey,
    MissingKey,
    Value(Difference),
}

#[derive(Debug)]
pub struct ArrayElementDifferences(Vec<(usize, Difference)>);

impl Serialize for ArrayElementDifferences {
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
pub struct ArrayLengthDifference(usize, usize);

#[derive(Debug, Serialize)]
pub enum ArrayDifference {
    LengthMismatch(ArrayLengthDifference),
    MismatchingElements(ArrayElementDifferences),
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
#[serde(untagged)]
pub enum ValueMismatch {
    Bool(bool, bool),
    String(String, String),
    Number(serde_json::Number, serde_json::Number),
}

#[derive(Debug)]
pub struct ObjectEntryDifferences(Vec<(String, EntryDifference)>);

impl Serialize for ObjectEntryDifferences {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, diff) in &self.0 {
            match diff {
                diff @ (EntryDifference::ExtraKey | EntryDifference::MissingKey) => {
                    map.serialize_entry(key, diff)?;
                }
                EntryDifference::Value(diff) => {
                    map.serialize_entry(key, diff)?;
                }
            }
        }
        map.end()
    }
}

#[derive(Debug, Serialize)]
pub enum Difference {
    ValueMismatch(ValueMismatch),
    TypeMismatch(Type, Type),
    Array(ArrayDifference),
    Object(ObjectEntryDifferences),
}

#[must_use]
pub fn arrays(a: Vec<serde_json::Value>, b: Vec<serde_json::Value>) -> Option<ArrayDifference> {
    let a_len = a.len();
    let b_len = b.len();

    if a_len != b_len {
        return Some(ArrayDifference::LengthMismatch(ArrayLengthDifference(
            a_len, b_len,
        )));
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
        Some(ArrayDifference::MismatchingElements(
            ArrayElementDifferences(element_differences),
        ))
    }
}

#[must_use]
pub fn objects(
    a: serde_json::Map<String, serde_json::Value>,
    mut b: serde_json::Map<String, serde_json::Value>,
) -> Option<ObjectEntryDifferences> {
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
        Some(ObjectEntryDifferences(value_differences))
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
                Some(Difference::ValueMismatch(ValueMismatch::Bool(a, b)))
            }
        }
        (Number(a), Number(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::ValueMismatch(ValueMismatch::Number(a, b)))
            }
        }
        (String(a), String(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::ValueMismatch(ValueMismatch::String(a, b)))
            }
        }
        (Array(a), Array(b)) => arrays(a, b).map(Difference::Array),
        (Object(a), Object(b)) => objects(a, b).map(Difference::Object),
        (a, b) => Some(Difference::TypeMismatch(a.into(), b.into())),
    }
}
