use serde_json::json;

/*
 MissingKey
 ExtraKey
 ValueMismatch
 TypeMismatch
 LengthMismatch
 MismatchingElements
*/
#[test]
fn kitchen_sink() {
    let a = json!({
      "matches": "a",
      "missing_key": "a",
      "value_mismatch": 1,
      "type_mismatch": 1,
      "length_mismatch": [],
      "mismatching_elements": ["a", "a"],
    });
    let b = json!({
      "matches": "a",
      "extra_key": "b",
      "value_mismatch": 2,
      "type_mismatch": "1",
      "length_mismatch": [true],
      "mismatching_elements": ["a", "ab"],
    });

    let diff = serde_json_compare::values(a, b);

    let diff = serde_json::to_value(diff).expect("couldn't serialize diff");

    insta::assert_snapshot!(serde_json::to_string_pretty(&diff).expect("couldn't pretty"));
}
