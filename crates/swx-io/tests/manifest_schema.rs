//! Checks the published JSON Schema for `manifest.json`, format v0.1
//! (`docs/specs/schema/manifest-0.1.json`), against the reference reader
//! [`swx_io::read_manifest`] and the golden manifest. No JSON Schema
//! validator is used: the schema's keywords are compared with the reader's
//! behaviour directly.

use std::path::PathBuf;

use serde_json::{Map, Value};
use swx_io::{read_manifest, to_canonical_json};

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn schema_text() -> String {
    std::fs::read_to_string(repo_path("../../docs/specs/schema/manifest-0.1.json"))
        .expect("schema file is readable")
}

fn schema() -> Value {
    serde_json::from_str(&schema_text()).expect("schema parses as JSON")
}

fn golden() -> Map<String, Value> {
    let text = std::fs::read_to_string(repo_path("tests/golden/manifest-v0.1.json"))
        .expect("golden manifest is readable");
    match serde_json::from_str(&text).expect("golden manifest parses") {
        Value::Object(map) => map,
        other => panic!("golden manifest is not an object: {other}"),
    }
}

fn string_set(v: &Value) -> Vec<String> {
    let mut out: Vec<String> = v
        .as_array()
        .expect("an array")
        .iter()
        .map(|s| s.as_str().expect("a string").to_string())
        .collect();
    out.sort();
    out
}

#[test]
fn schema_is_canonical_json_draft_2020_12() {
    let text = schema_text();
    let value = schema();
    assert_eq!(text, to_canonical_json(&value));
    assert_eq!(
        value["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert!(value["title"].is_string());
    assert!(value["description"].is_string());
    assert_eq!(value["type"], "object");
    assert_ne!(value["additionalProperties"], Value::Bool(false));
    assert_ne!(
        value["properties"]["units"]["additionalProperties"],
        Value::Bool(false)
    );
}

#[test]
fn golden_manifest_is_accepted() {
    let text = to_canonical_json(&Value::Object(golden()));
    read_manifest(&text).expect("golden manifest reads");
}

#[test]
fn required_matches_what_read_manifest_rejects_when_missing() {
    let schema = schema();
    let required = string_set(&schema["required"]);
    let golden = golden();
    for key in golden.keys() {
        let mut m = golden.clone();
        m.remove(key);
        let fails = read_manifest(&to_canonical_json(&Value::Object(m))).is_err();
        assert_eq!(
            fails,
            required.contains(key),
            "removing {key:?}: read_manifest fails = {fails}, schema requires = {}",
            required.contains(key)
        );
    }
    for key in &required {
        assert!(golden.contains_key(key), "required {key:?} not in golden");
    }
}

#[test]
fn units_required_matches_what_read_manifest_rejects_when_missing() {
    let schema = schema();
    let required = string_set(&schema["properties"]["units"]["required"]);
    let golden = golden();
    let units = golden["units"].as_object().expect("units is an object");
    for key in units.keys() {
        let mut m = golden.clone();
        m["units"].as_object_mut().unwrap().remove(key);
        let fails = read_manifest(&to_canonical_json(&Value::Object(m))).is_err();
        assert_eq!(fails, required.contains(key), "removing units.{key}");
    }
}

#[test]
fn every_required_and_golden_key_has_a_property() {
    let schema = schema();
    let properties = schema["properties"].as_object().expect("properties");
    for key in string_set(&schema["required"]) {
        assert!(properties.contains_key(&key), "required {key:?}");
    }
    for key in golden().keys() {
        assert!(properties.contains_key(key), "golden key {key:?}");
    }
    for field in ["format_version", "document_id", "model_hash"] {
        assert_eq!(properties[field]["type"], "string", "{field}");
        assert!(properties[field]["pattern"].is_string(), "{field}");
        assert!(properties[field]["description"].is_string(), "{field}");
    }
}

#[test]
fn kind_enum_and_format_const_match_read_manifest() {
    let schema = schema();
    let kind = &schema["properties"]["kind"]["enum"];
    let mut expected = vec!["assembly", "drawing", "part"];
    expected.sort();
    assert_eq!(string_set(kind), expected);
    for name in string_set(kind) {
        let mut m = golden();
        m.insert("kind".to_string(), Value::String(name.clone()));
        read_manifest(&to_canonical_json(&Value::Object(m)))
            .unwrap_or_else(|e| panic!("kind {name:?} rejected: {e}"));
    }
    let mut m = golden();
    m.insert("kind".to_string(), Value::String("sheet".to_string()));
    assert!(read_manifest(&to_canonical_json(&Value::Object(m))).is_err());

    assert_eq!(schema["properties"]["format"]["const"], "sltwx");
}
