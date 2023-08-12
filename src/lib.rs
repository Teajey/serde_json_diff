#![doc = include_str!("../README.md")]
use serde::{ser::SerializeMap, Serialize};

#[derive(Debug, Serialize)]
#[serde(tag = "entry_difference")]
pub enum EntryDifference {
    Extra { value: serde_json::Value },
    Missing,
    Deep { diff: Difference },
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
#[serde(tag = "array_difference")]
pub enum ArrayDifference {
    PairsOnly {
        different_pairs: DumbMap<usize, Difference>,
    },
    Shorter {
        different_pairs: Option<DumbMap<usize, Difference>>,
        extra_elements: Vec<serde_json::Value>,
    },
    Longer {
        different_pairs: Option<DumbMap<usize, Difference>>,
        missing_elements: usize,
    },
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
pub enum ScalarDifference {
    Bool {
        source: bool,
        target: bool,
    },
    String {
        source: String,
        target: String,
    },
    Number {
        source: serde_json::Number,
        target: serde_json::Number,
    },
}

#[derive(Debug, Serialize)]
pub struct TypeDifference {
    source_type: Type,
    target_type: Type,
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
// FIXME: This feels pretty overwrought compared to `objects` and `values`. Maybe there's a better way to diff arrays...
pub fn arrays(
    source: Vec<serde_json::Value>,
    target: Vec<serde_json::Value>,
) -> Option<ArrayDifference> {
    let mut source_iter = source.into_iter().enumerate().peekable();
    let mut target_iter = target.into_iter().peekable();

    let mut different_pairs = vec![];
    while let (Some(_), Some(_)) = (source_iter.peek(), target_iter.peek()) {
        let ((i, source), target) = match (source_iter.next(), target_iter.next()) {
            (Some(source), Some(target)) => (source, target),
            _ => unreachable!("checked by peek()"),
        };
        different_pairs.push(values(source, target).map(|diff| (i, diff)));
    }
    let different_pairs = different_pairs.into_iter().flatten().collect::<Vec<_>>();
    let different_pairs = if different_pairs.is_empty() {
        None
    } else {
        Some(DumbMap(different_pairs))
    };

    let extra_elements = source_iter.map(|(_, source)| source).collect::<Vec<_>>();
    let missing_elements = target_iter.collect::<Vec<_>>();

    if !extra_elements.is_empty() {
        return Some(ArrayDifference::Shorter {
            different_pairs,
            extra_elements,
        });
    }

    if !missing_elements.is_empty() {
        return Some(ArrayDifference::Longer {
            different_pairs,
            missing_elements: missing_elements.len(),
        });
    }

    if let Some(different_pairs) = different_pairs {
        Some(ArrayDifference::PairsOnly { different_pairs })
    } else {
        None
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
                return Some((key, EntryDifference::Missing));
            };

            values(a, b).map(|diff| (key, EntryDifference::Deep { diff }))
        })
        .collect::<Vec<_>>();

    value_differences.extend(b.into_iter().map(|(extra_key, extra_value)| {
        (extra_key, EntryDifference::Extra { value: extra_value })
    }));

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
        (Bool(source), Bool(target)) => {
            if source == target {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::Bool {
                    source,
                    target,
                }))
            }
        }
        (Number(source), Number(target)) => {
            if source == target {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::Number {
                    source,
                    target,
                }))
            }
        }
        (String(source), String(target)) => {
            if source == target {
                None
            } else {
                Some(Difference::Scalar(ScalarDifference::String {
                    source,
                    target,
                }))
            }
        }
        (Array(a), Array(b)) => arrays(a, b).map(Difference::Array),
        (Object(a), Object(b)) => objects(a, b).map(Difference::Object),
        (a, b) => Some(Difference::Type(TypeDifference {
            source_type: a.into(),
            target_type: b.clone().into(),
            target_value: b,
        })),
    }
}
