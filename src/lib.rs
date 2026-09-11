//! JSON ↔ KVD conversion.
//!
//! Semantics (spec §5):
//! - JSON nulls become the `null` literal.
//! - Empty containers become `{}` / `[]`.
//! - Strings colliding with KVD shapes are quoted on emit.
//! - Multi-line strings become `"""` blocks.
//! - Non-finite floats (`nan`, `inf`) are rejected.

#![warn(missing_docs)]

use kvd_rs::value::{Map, Node, Shape};
use serde_json::Value;
use std::fmt;

/// Conversion error (e.g. non-finite floats).
#[derive(Debug, Clone)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// Converts a JSON value to a KVD node.
pub fn from_json(v: &Value) -> Result<Node, Error> {
    match v {
        Value::Null => Ok(Node::scalar(Shape::Null, "null")),
        Value::Bool(b) => Ok(Node::scalar(Shape::Bool, b.to_string())),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Node::scalar(Shape::Int, i.to_string()))
            } else if let Some(u) = n.as_u64() {
                Ok(Node::scalar(Shape::Int, u.to_string()))
            } else if let Some(f) = n.as_f64() {
                if f.is_finite() {
                    Ok(Node::scalar(Shape::Float, fmt_float(f)))
                } else {
                    Err(Error(format!(
                        "non-finite float {f} has no KVD representation"
                    )))
                }
            } else {
                Err(Error("unsupported JSON number".to_string()))
            }
        }
        Value::String(s) => Ok(Node::scalar(Shape::Str, s.clone())),
        Value::Array(arr) => {
            let items: Result<Vec<_>, _> = arr.iter().map(from_json).collect();
            Ok(Node::list(items?))
        }
        Value::Object(map) => {
            let mut m = Map::new();
            for (k, v) in map {
                m.insert(k.clone(), from_json(v)?);
            }
            Ok(Node::map(m))
        }
    }
}

/// Formats an f64 as a KVD float literal.
fn fmt_float(f: f64) -> String {
    let s = format!("{f}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{f:.1}")
    }
}

/// Converts a KVD node to a JSON value tree.
pub fn to_json_value(node: &Node) -> Result<Value, Error> {
    match node {
        Node::Scalar(s) => match s.shape {
            Shape::Bool => Ok(Value::Bool(s.text == "true")),
            Shape::Int => {
                let clean: String = s.text.chars().filter(|c| *c != '_').collect();
                if let Ok(i) = clean.parse::<i64>() {
                    Ok(Value::Number(i.into()))
                } else if let Ok(u) = clean.parse::<u64>() {
                    Ok(Value::Number(u.into()))
                } else {
                    Err(Error(format!(
                        "int literal `{}` out of range for JSON",
                        s.text
                    )))
                }
            }
            Shape::Float => {
                let f: f64 = s
                    .text
                    .parse()
                    .map_err(|_| Error(format!("invalid float literal `{}`", s.text)))?;
                if !f.is_finite() {
                    return Err(Error(format!(
                        "float literal `{}` has no JSON representation",
                        s.text
                    )));
                }
                // serde_json::Number::from_f64 returns Option, None if non-finite (already checked)
                Ok(Value::Number(serde_json::Number::from_f64(f).ok_or_else(
                    || {
                        Error(format!(
                            "float literal `{}` has no JSON representation",
                            s.text
                        ))
                    },
                )?))
            }
            Shape::Str => Ok(Value::String(s.text.clone())),
            Shape::Null => Ok(Value::Null),
        },
        Node::Map(m) | Node::Dict(m) => {
            let mut out = serde_json::Map::new();
            for (k, v) in m.iter() {
                out.insert(k.to_string(), to_json_value(v)?);
            }
            Ok(Value::Object(out))
        }
        Node::List(items) => {
            let items: Result<Vec<_>, _> = items.iter().map(to_json_value).collect();
            Ok(Value::Array(items?))
        }
        _ => Err(Error("unsupported node kind".to_string())),
    }
}

/// Serializes a KVD node as JSON text (pretty-printed).
pub fn to_json(node: &Node) -> Result<String, Error> {
    let v = to_json_value(node)?;
    serde_json::to_string_pretty(&v).map_err(|e| Error(e.to_string()))
}

/// Converts KVD text to JSON text.
pub fn kvd_text_to_json(kvd_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let node = kvd_rs::deserialize::from_str(kvd_text)?;
    Ok(to_json(&node)?)
}

/// Converts JSON text to KVD text, optionally verifying against a JSON schema.
///
/// The schema (if provided) is itself a JSON file that mirrors the data
/// structure — same format as a KVD schema document but written in JSON.
/// It is converted to a KVD node via [`from_json`] before verification.
pub fn json_text_to_kvd(
    json_text: &str,
    schema_text: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_str(json_text)?;
    let node = from_json(&value)?;
    if let Some(schema_text) = schema_text {
        let schema_value: Value = serde_json::from_str(schema_text)?;
        let schema_doc = from_json(&schema_value)?;
        if let Err(e) = kvd_rs::schema::verify(&node, &schema_doc) {
            let violations = match &e {
                kvd_rs::schema::VerifyError::Violations(v)
                | kvd_rs::schema::VerifyError::SchemaMalformed(v) => v,
                other => return Err(other.to_string().into()),
            };
            for v in violations {
                eprintln!("{v}");
            }
            let label = if violations.len() == 1 {
                "schema error"
            } else {
                "schema errors"
            };
            return Err(format!("{} {label}", violations.len()).into());
        }
    }
    Ok(kvd_rs::serialize::to_string(&node)?)
}
