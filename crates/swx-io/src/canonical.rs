//! The canonical JSON writer used by every `.sltwx` file
//! (`docs/specs/FILE_FORMAT.md`, "Unpacked (git-friendly) form").

use serde::Serialize;
use serde_json::Value;

/// Writes `v` as canonical JSON (RFC 8259, UTF-8):
///
/// - object keys in ascending byte order at every level;
/// - 2-space indent, one object member and one array element per line;
/// - `": "` between a key and its value, `,` at the end of every line but
///   the last of an object or array;
/// - empty objects and arrays written as `{}` and `[]`;
/// - strings escaped as `serde_json` does (quotes, backslashes and control
///   characters only; other characters stay as UTF-8);
/// - exactly one trailing `\n`.
///
/// The same value always gives the same bytes, whatever order its keys were
/// built in, so version-control diffs of the unpacked form stay small.
///
/// # Panics
///
/// If `v` cannot be represented as JSON: its `Serialize` implementation
/// fails, or it is a map whose keys are not strings. A
/// [`serde_json::Value`], which is what this crate writes, never panics.
pub fn to_canonical_json<T: Serialize>(v: &T) -> String {
    let value = serde_json::to_value(v).expect("value must be representable as JSON");
    let mut out = String::new();
    write_value(&mut out, &value, 0);
    out.push('\n');
    out
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

fn write_string(out: &mut String, s: &str) {
    // Serializing a `str` never fails.
    out.push_str(&serde_json::to_string(s).unwrap_or_default());
}

fn write_value(out: &mut String, v: &Value, level: usize) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => write_string(out, s),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                indent(out, level + 1);
                write_value(out, item, level + 1);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            indent(out, level);
            out.push(']');
        }
        Value::Object(map) if map.is_empty() => out.push_str("{}"),
        Value::Object(map) => {
            // Sort explicitly: the map's own order depends on serde_json's
            // features, which another crate in the build could change.
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            out.push_str("{\n");
            for (i, (key, value)) in entries.iter().enumerate() {
                indent(out, level + 1);
                write_string(out, key);
                out.push_str(": ");
                write_value(out, value, level + 1);
                if i + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            indent(out, level);
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scalars_and_empty_containers() {
        assert_eq!(to_canonical_json(&json!(null)), "null\n");
        assert_eq!(to_canonical_json(&json!(true)), "true\n");
        assert_eq!(to_canonical_json(&json!(-2.5)), "-2.5\n");
        assert_eq!(to_canonical_json(&json!("a\"b\nü")), "\"a\\\"b\\nü\"\n");
        assert_eq!(to_canonical_json(&json!({})), "{}\n");
        assert_eq!(to_canonical_json(&json!([])), "[]\n");
    }

    #[test]
    fn keys_sorted_by_bytes_at_every_level_and_one_element_per_line() {
        let v = json!({
            "b": [1, {"z": 0, "a": []}],
            "B": "upper sorts first",
            "a": {"y": 1, "x": {}},
        });
        let expected = "{\n  \"B\": \"upper sorts first\",\n  \"a\": {\n    \"x\": {},\n    \"y\": 1\n  },\n  \"b\": [\n    1,\n    {\n      \"a\": [],\n      \"z\": 0\n    }\n  ]\n}\n";
        assert_eq!(to_canonical_json(&v), expected);
    }

    #[test]
    fn same_value_same_bytes_whatever_the_input_order() {
        let a: Value =
            serde_json::from_str(r#"{"k": 1, "j": [2, 3], "i": {"q": 1, "p": 2}}"#).unwrap();
        let b: Value =
            serde_json::from_str(r#"{"i": {"p": 2, "q": 1}, "j": [2, 3], "k": 1}"#).unwrap();
        assert_eq!(to_canonical_json(&a), to_canonical_json(&b));
        assert!(to_canonical_json(&a).ends_with("}\n"));
        assert!(!to_canonical_json(&a).ends_with("\n\n"));
    }
}
