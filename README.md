Create machine-readable JSON diffs

## Usage

### Library

```rust
let a = serde_json::json!({
  "list": [1, 2, 3],
  "object": {"a": "b"}
});

let b = serde_json::json!({
  "list": [1, 2, 3],
  "object": {"a": "b"}
});

assert!(serde_json_diff::values(a, b).is_none());
```

`serde_json_diff::objects` and `serde_json_diff::arrays` are also exposed
specifically for comparing `serde_json::Map<String, serde_json::Value>`
and `Vec<serde_json::Value>`s respectively.

### CLI

```sh
serde_json_diff my_json_file.json my_other_json_file.json
```

Tip: Since the command name `serde_json_diff` is a bit long, I personally have it aliased in my shell config:
```sh
alias jdiff="serde_json_diff"
```

## Example

Comparing this file:
```json
{
  "A": "a",
  "B": "a",
  "D": 1,
  "E": 1,
  "F": [],
  "G": ["a", "a"]
}
```
To this file:
```json
{
  "A": "a",
  "C": "b",
  "D": 2,
  "E": "1",
  "F": [true],
  "G": ["a", "ab"]
}
```
Results in this diff (serialised as JSON):
```json
{
  "B": {
    "entry_difference": "extra"
  },
  "D": {
    "entry_difference": "value",
    "value_diff": {
      "difference_of": "scalar",
      "source": 1,
      "target": 2
    }
  },
  "E": {
    "entry_difference": "value",
    "value_diff": {
      "difference_of": "type",
      "source_type": "number",
      "target_type": "string",
      "target_value": "1"
    }
  },
  "F": {
    "entry_difference": "value",
    "value_diff": {
      "difference_of": "array",
      "array_difference": "longer",
      "different_pairs": null,
      "missing_elements": 1
    }
  },
  "G": {
    "entry_difference": "value",
    "value_diff": {
      "difference_of": "array",
      "array_difference": "pairs_only",
      "different_pairs": {
        "1": {
          "difference_of": "scalar",
          "source": "a",
          "target": "ab"
        }
      }
    }
  },
  "C": {
    "entry_difference": "missing",
    "value": "b"
  }
}
```
