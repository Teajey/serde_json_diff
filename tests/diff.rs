use serde_json::json;

#[test]
fn kitchen_sink() {
    let a = json!({
      "matches": "a",
      "missing_key": "a",
      "value_difference": 1,
      "type_difference": 1,
      "length_difference": [],
      "different_elements": ["a", "a"],
    });
    let b = json!({
      "matches": "a",
      "extra_key": "b",
      "value_difference": 2,
      "type_difference": "1",
      "length_difference": [true],
      "different_elements": ["a", "ab"],
    });

    let diff = serde_json_diff::values(a, b);

    insta::assert_snapshot!(serde_json::to_string_pretty(&diff).expect("couldn't pretty"));
}

#[test]
fn types() {
    let left = json!("a");
    let right = json!(true);

    let diff = serde_json_diff::values(left, right);

    insta::assert_snapshot!(serde_json::to_string_pretty(&diff).unwrap());
}

#[test]
fn entries() {
    let left = json!({
        "a": false,
        "c": 1,
    });
    let right = json!({
        "b": false,
        "c": 2,
    });

    let diff = serde_json_diff::objects(
        serde_json::from_value(left).unwrap(),
        serde_json::from_value(right).unwrap(),
    );

    insta::assert_snapshot!(serde_json::to_string_pretty(&diff).unwrap());
}
