//! Checks the published JSON Schema for a part's `model.json`, format v0.1
//! (`docs/specs/schema/model-part-0.1.json`), against the golden part and
//! the reference reader [`swx_io::read_unpacked`]. No JSON Schema validator
//! is used: a small checker below evaluates only the `properties`, `items`,
//! `required`, `type` and `minLength` keywords (JSON Schema draft 2020-12,
//! "JSON Schema Validation", sections 6.1.1, 6.3.1 and 6.5.3, and "JSON
//! Schema Core", sections 10.3.1.2 and 10.3.2.1).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{json, Value};
use swx_io::{read_unpacked, to_canonical_json};

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn schema_text() -> String {
    fs::read_to_string(repo_path("../../docs/specs/schema/model-part-0.1.json"))
        .expect("schema file is readable")
}

fn schema() -> Value {
    serde_json::from_str(&schema_text()).expect("schema parses as JSON")
}

fn golden_dir() -> PathBuf {
    repo_path("tests/golden/bracket.sltwx")
}

fn golden_model() -> Value {
    let text = fs::read_to_string(golden_dir().join("model.json")).expect("golden model");
    serde_json::from_str(&text).expect("golden model parses")
}

/// Whether `v` has the JSON Schema `type` named `t`.
fn has_type(v: &Value, t: &str) -> bool {
    match t {
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "number" => v.is_number(),
        "integer" => v.is_i64() || v.is_u64(),
        "boolean" => v.is_boolean(),
        "null" => v.is_null(),
        other => panic!("type {other:?} is not handled by this checker"),
    }
}

/// Checks `v` against `schema` using only `type`, `minLength`, `required`,
/// `properties` and `items`. Returns the JSON pointer of the first failure.
fn check(schema: &Value, v: &Value, at: &str) -> Result<(), String> {
    if let Some(t) = schema.get("type").and_then(Value::as_str) {
        if !has_type(v, t) {
            return Err(format!("{at}: not of type {t}"));
        }
    }
    if let (Some(min), Some(s)) = (schema.get("minLength").and_then(Value::as_u64), v.as_str()) {
        if (s.chars().count() as u64) < min {
            return Err(format!("{at}: shorter than {min}"));
        }
    }
    if let Some(obj) = v.as_object() {
        for key in schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let key = key.as_str().expect("required names are strings");
            if !obj.contains_key(key) {
                return Err(format!("{at}: missing {key:?}"));
            }
        }
        if let Some(props) = schema.get("properties").and_then(Value::as_object) {
            for (key, sub) in props {
                if let Some(child) = obj.get(key) {
                    check(sub, child, &format!("{at}/{key}"))?;
                }
            }
        }
    }
    if let (Some(items), Some(arr)) = (schema.get("items"), v.as_array()) {
        for (i, child) in arr.iter().enumerate() {
            check(items, child, &format!("{at}/{i}"))?;
        }
    }
    Ok(())
}

/// A copy of the golden part folder with `model` as its `model.json`,
/// removed when dropped.
struct PartCopy(PathBuf);

impl PartCopy {
    fn new(model: &Value) -> PartCopy {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("swx-io-model-schema-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let dir = root.join("bracket.sltwx");
        fs::create_dir_all(&dir).unwrap();
        for name in ["manifest.json", "properties.json"] {
            fs::copy(golden_dir().join(name), dir.join(name)).unwrap();
        }
        fs::write(dir.join("model.json"), to_canonical_json(model)).unwrap();
        PartCopy(root)
    }

    fn dir(&self) -> PathBuf {
        self.0.join("bracket.sltwx")
    }
}

impl Drop for PartCopy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn reads(model: &Value) -> bool {
    read_unpacked(&PartCopy::new(model).dir()).is_ok()
}

/// Walks every subschema reachable through `properties` and `items`.
fn walk<'a>(schema: &'a Value, out: &mut Vec<(String, &'a Value)>, at: String) {
    out.push((at.clone(), schema));
    if let Some(props) = schema.get("properties").and_then(Value::as_object) {
        for (k, sub) in props {
            walk(sub, out, format!("{at}/properties/{k}"));
        }
    }
    if let Some(items) = schema.get("items") {
        walk(items, out, format!("{at}/items"));
    }
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
    assert!(value.get("required").is_none());
    let mut all = Vec::new();
    walk(&value, &mut all, String::new());
    for (at, sub) in &all {
        assert_ne!(
            sub.get("additionalProperties"),
            Some(&Value::Bool(false)),
            "{at}"
        );
    }
    assert_eq!(value["properties"]["end_of_part"]["type"], "string");
}

#[test]
fn the_five_id_arrays_require_a_non_empty_string_id() {
    let s = schema();
    let p = &s["properties"];
    let sk = &p["sketches"]["items"]["properties"];
    for (name, arr) in [
        ("sketches", &p["sketches"]),
        ("features", &p["features"]),
        ("bodies", &p["bodies"]),
        ("entities", &sk["entities"]),
        ("constraints", &sk["constraints"]),
    ] {
        assert_eq!(arr["type"], "array", "{name}");
        assert_eq!(arr["items"]["type"], "object", "{name}");
        assert_eq!(arr["items"]["required"], json!(["id"]), "{name}");
        assert_eq!(
            arr["items"]["properties"]["id"],
            json!({"minLength": 1, "type": "string"}),
            "{name}"
        );
        let d = arr["description"].as_str().expect("a description");
        assert!(d.contains("unique"), "{name}: {d}");
        if name == "features" {
            assert!(d.contains("not sorted"), "{name}: {d}");
        } else {
            assert!(d.contains("sorted by `id`"), "{name}: {d}");
        }
    }
    assert_eq!(p["parameters"]["type"], "array");
}

#[test]
fn every_golden_top_level_key_has_a_property() {
    let s = schema();
    let props = s["properties"].as_object().unwrap();
    for key in golden_model().as_object().unwrap().keys() {
        assert!(props.contains_key(key), "{key} has no schema property");
    }
}

#[test]
fn golden_model_passes_the_checker_and_reads() {
    let model = golden_model();
    assert_eq!(check(&schema(), &model, ""), Ok(()));
    assert!(reads(&model));
}

#[test]
fn bad_ids_fail_the_checker_and_read_unpacked() {
    let s = schema();
    let mut no_entity_id = golden_model();
    no_entity_id["sketches"][0]["entities"][0]
        .as_object_mut()
        .unwrap()
        .remove("id");
    let mut numeric_body_id = golden_model();
    numeric_body_id["bodies"][0]["id"] = json!(5);
    let mut empty_feature_id = golden_model();
    empty_feature_id["features"][0]["id"] = json!("");
    for (name, model, pointer) in [
        ("entity without id", no_entity_id, "/sketches/0/entities/0"),
        ("numeric body id", numeric_body_id, "/bodies/0/id"),
        ("empty feature id", empty_feature_id, "/features/0/id"),
    ] {
        let err = check(&s, &model, "").expect_err(name);
        assert!(err.starts_with(pointer), "{name}: {err}");
        assert!(!reads(&model), "{name}: read_unpacked accepted it");
    }
}

#[test]
fn duplicate_body_ids_pass_the_checker_but_not_read_unpacked() {
    let mut model = golden_model();
    let body = model["bodies"][0].clone();
    model["bodies"].as_array_mut().unwrap().push(body);
    assert_eq!(check(&schema(), &model, ""), Ok(()));
    assert!(!reads(&model));
}

#[test]
fn checker_sees_a_wrong_top_level_type() {
    let mut model = golden_model();
    model["end_of_part"] = json!(7);
    let err = check(&schema(), &model, "").unwrap_err();
    assert!(err.starts_with("/end_of_part"), "{err}");
}
