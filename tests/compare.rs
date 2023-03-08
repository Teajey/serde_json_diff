use serde_json::json;

#[test]
fn objects() {
    let a = json!({
      "1": "2",
      "3": 4,
      "4": [4, "5"],
      "5": [4, "5"],
      "-20": [4, "5"],
      "numbero": 1
    })
    .as_object()
    .unwrap()
    .clone();
    let b = json!({
      "1": "2",
      "3": "4",
      "4": [4, "6", 7],
      "5": [4, "7"],
      "100": [4, "7"],
      "numbero": 7
    })
    .as_object()
    .unwrap()
    .clone();

    let diff = serde_json_compare::objects(a, b);

    let diff = serde_json::to_value(diff).expect("couldn't serialize diff");

    insta::assert_snapshot!(serde_json::to_string_pretty(&diff).expect("couldn't pretty"));
}
