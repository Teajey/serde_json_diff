Create machine-readable JSON diffs

## Usage

### Library

```rust
# use serde_json::json;

let a = json!({
  "list": [1, 2, 3],
  "object": {"a": "b"}
});

let b = json!({
  "list": [1, 2, 3],
  "object": {"a": "b"}
});

assert!(serde_json_diff::values(a, b).is_none());
```

[`serde_json_diff::objects`](objects) and [`serde_json_diff::arrays`](arrays) are also exposed
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
  "matches": "a",
  "missing_key": "a",
  "value_difference": 1,
  "type_difference": 1,
  "length_difference": [],
  "different_elements": ["a", "a"]
}
```
To this file:
```json
{
  "matches": "a",
  "extra_key": "b",
  "value_difference": 2,
  "type_difference": "1",
  "length_difference": [true],
  "different_elements": ["a", "ab"]
}
```
Results in this diff ([`Difference`] type serialised as JSON):
```json
{
  "Object": {
    "different_elements": {
      "Value": {
        "Array": {
          "Element": {
            "1": {
              "Scalar": [
                "a",
                "ab"
              ]
            }
          }
        }
      }
    },
    "extra_key": "ExtraKey",
    "length_difference": {
      "Value": {
        "Array": {
          "Length": [
            0,
            1
          ]
        }
      }
    },
    "missing_key": "MissingKey",
    "type_difference": {
      "Value": {
        "Type": [
          "Number",
          "String"
        ]
      }
    },
    "value_difference": {
      "Value": {
        "Scalar": [
          1,
          2
        ]
      }
    }
  }
}
```
Admittedly, the output is not particularly human-readable or intuitive in JSON form. So I'm very open to suggestions on how this can be improved! 😇