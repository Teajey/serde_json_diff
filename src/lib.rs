use serde::{
    ser::{SerializeMap, SerializeTuple},
    Serialize,
};

#[derive(Debug, Serialize)]
pub enum Leaf {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
}

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

#[derive(Debug)]
pub enum ValueMismatch {
    Bool(bool, bool),
    String(String, String),
    Number(serde_json::Number, serde_json::Number),
}

impl Serialize for ValueMismatch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        match self {
            ValueMismatch::Bool(a, b) => {
                tuple.serialize_element(a)?;
                tuple.serialize_element(b)?;
            }
            ValueMismatch::String(a, b) => {
                tuple.serialize_element(a)?;
                tuple.serialize_element(b)?;
            }
            ValueMismatch::Number(a, b) => {
                tuple.serialize_element(a)?;
                tuple.serialize_element(b)?;
            }
        }
        tuple.end()
    }
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

#[must_use]
pub fn values(a: serde_json::Value, b: serde_json::Value) -> Option<Difference> {
    use serde_json::Value::{Array, Bool, Null, Number, Object, String};

    match (a, b) {
        (Null, Null) => None,
        (Null, Bool(_)) => Some(Difference::TypeMismatch(Type::Null, Type::Bool)),
        (Null, Number(_)) => Some(Difference::TypeMismatch(Type::Null, Type::Number)),
        (Null, String(_)) => Some(Difference::TypeMismatch(Type::Null, Type::String)),
        (Null, Array(_)) => Some(Difference::TypeMismatch(Type::Null, Type::Array)),
        (Null, Object(_)) => Some(Difference::TypeMismatch(Type::Null, Type::Object)),
        (Bool(_), Null) => Some(Difference::TypeMismatch(Type::Bool, Type::Null)),
        (Bool(a), Bool(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::ValueMismatch(ValueMismatch::Bool(a, b)))
            }
        }
        (Bool(_), Number(_)) => Some(Difference::TypeMismatch(Type::Bool, Type::Number)),
        (Bool(_), String(_)) => Some(Difference::TypeMismatch(Type::Bool, Type::String)),
        (Bool(_), Array(_)) => Some(Difference::TypeMismatch(Type::Bool, Type::Array)),
        (Bool(_), Object(_)) => Some(Difference::TypeMismatch(Type::Bool, Type::Object)),
        (Number(_), Null) => Some(Difference::TypeMismatch(Type::Number, Type::Null)),
        (Number(_), Bool(_)) => Some(Difference::TypeMismatch(Type::Number, Type::Bool)),
        (Number(a), Number(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::ValueMismatch(ValueMismatch::Number(a, b)))
            }
        }
        (Number(_), String(_)) => Some(Difference::TypeMismatch(Type::Number, Type::String)),
        (Number(_), Array(_)) => Some(Difference::TypeMismatch(Type::Number, Type::Array)),
        (Number(_), Object(_)) => Some(Difference::TypeMismatch(Type::Number, Type::Object)),
        (String(_), Null) => Some(Difference::TypeMismatch(Type::String, Type::Null)),
        (String(_), Bool(_)) => Some(Difference::TypeMismatch(Type::String, Type::Bool)),
        (String(_), Number(_)) => Some(Difference::TypeMismatch(Type::String, Type::Number)),
        (String(a), String(b)) => {
            if a == b {
                None
            } else {
                Some(Difference::ValueMismatch(ValueMismatch::String(a, b)))
            }
        }
        (String(_), Array(_)) => Some(Difference::TypeMismatch(Type::String, Type::Array)),
        (String(_), Object(_)) => Some(Difference::TypeMismatch(Type::String, Type::Object)),
        (Array(_), Null) => Some(Difference::TypeMismatch(Type::Array, Type::Null)),
        (Array(_), Bool(_)) => Some(Difference::TypeMismatch(Type::Array, Type::Bool)),
        (Array(_), Number(_)) => Some(Difference::TypeMismatch(Type::Array, Type::Number)),
        (Array(_), String(_)) => Some(Difference::TypeMismatch(Type::Array, Type::String)),
        (Array(a), Array(b)) => arrays(a, b).map(Difference::Array),
        (Array(_), Object(_)) => Some(Difference::TypeMismatch(Type::Array, Type::Object)),
        (Object(_), Null) => Some(Difference::TypeMismatch(Type::Object, Type::Null)),
        (Object(_), Bool(_)) => Some(Difference::TypeMismatch(Type::Object, Type::Bool)),
        (Object(_), Number(_)) => Some(Difference::TypeMismatch(Type::Object, Type::Number)),
        (Object(_), String(_)) => Some(Difference::TypeMismatch(Type::Object, Type::String)),
        (Object(_), Array(_)) => Some(Difference::TypeMismatch(Type::Object, Type::Array)),
        (Object(a), Object(b)) => objects(a, b).map(Difference::Object),
    }
}
