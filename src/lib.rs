//! Infers JSON Type Definition schemas from example inputs.
//!
//! JSON Type Definition, aka [RFC 8927](https://tools.ietf.org/html/rfc8927),
//! is an easy-to-learn, standardized way to define a schema for JSON data. You
//! can use JSON Typedef to portably validate data across programming languages,
//! create dummy data, generate code, and more.
//!
//! This Rust crate can generate a JSON Typedef schema from example data. If you
//! are looking to use this package as a CLI tool, see [this crate's
//! README](https://github.com/jsontypedef/json-typedef-infer). The remainder of
//! these docs are focused on this crate as a Rust library, and so focuses on
//! the Rust API for using `jtd_fuzz`.
//!
//! # Quick start
//!
//! Here's how you can use this crate to infer a schema:
//!
//! ```
//! use serde_json::json;
//! use jtd_infer::{Inferrer, Hints, HintSet, NumType};
//!
//! let mut inferrer = Inferrer::new(Hints::new(
//!     NumType::Uint8,
//!     HintSet::new(vec![]),
//!     HintSet::new(vec![]),
//!     HintSet::new(vec![]),
//! ));
//!
//! inferrer = inferrer.infer(json!({ "foo": true, "bar": "xxx" }));
//! inferrer = inferrer.infer(json!({ "foo": false, "bar": null, "baz": 5 }));
//!
//! let inference = inferrer.into_schema();
//!
//! assert_eq!(
//!     json!({
//!         "properties": {
//!             "foo": { "type": "boolean" },
//!             "bar": { "type": "string", "nullable": true },
//!         },
//!         "optionalProperties": {
//!             "baz": { "type": "uint8" },
//!         },
//!     }),
//!     serde_json::to_value(inference.into_serde_schema()).unwrap(),
//! )
//! ```

mod hints;
mod inferred_number;
mod inferred_schema;

pub use crate::hints::{HintSet, Hints};
pub use crate::inferred_number::NumType;
use crate::inferred_schema::InferredSchema;
use jtd::Schema;
use serde_json::Value;

use anyhow::Error;
use clap::{crate_version, load_yaml, App, AppSettings};
use serde_json::Deserializer;
use std::fs::File;
use std::io::stdin;
use std::io::BufReader;
use std::io::Read;
use std::io::Cursor;

use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;

#[derive(Serialize, Deserialize)]
pub struct SchemaParams {
    input: String,
    enumHints: Vec<String>,
    valuesHints: Vec<String>,
    discriminatorHints: Vec<String>,
    defaultNumberType: String,
}


#[wasm_bindgen]
pub fn generate_schema(params_js: JsValue) -> Result<String, JsValue> {
//     let params: SchemaParams = params_js.into_serde().map_err(|e| JsValue::from_str(&e.to_string()))?;
    let params: SchemaParams = from_value(params_js).map_err(|e| JsValue::from_str(&e.to_string()))?;

//     let enum_hints: Vec<String> = serde_json::from_str(&enum_hints.as_string().unwrap()).map_err(|e| JsValue::from_str(&e.to_string()))?;
//     let values_hints: Vec<String> = serde_json::from_str(&values_hints.as_string().unwrap()).map_err(|e| JsValue::from_str(&e.to_string()))?;
//     let discriminator_hints: Vec<String> = serde_json::from_str(&discriminator_hints.as_string().unwrap()).map_err(|e| JsValue::from_str(&e.to_string()))?;


    let reader = BufReader::new(Cursor::new(params.input));

    let enum_hints: Vec<Vec<_>> = params.enumHints
        .iter()
        .map(|hint| parse_json_pointer(hint))
        .collect();

    let values_hints: Vec<Vec<_>> = params.valuesHints
        .iter()
        .map(|hint| parse_json_pointer(hint))
        .collect();

    let discriminator_hints: Vec<Vec<_>> = params.discriminatorHints
        .iter()
        .map(|hint| parse_json_pointer(hint))
        .collect();

    let default_num_type = match params.defaultNumberType.as_str() {
        "int8" => NumType::Int8,
        "uint8" => NumType::Uint8,
        "int16" => NumType::Int16,
        "uint16" => NumType::Uint16,
        "int32" => NumType::Int32,
        "uint32" => NumType::Uint32,
        "float32" => NumType::Float32,
        "float64" => NumType::Float64,
        _ => return Err(JsValue::from_str("Invalid default number type")),
    };


    let hints = Hints::new(
        default_num_type,
        HintSet::new(enum_hints.iter().map(|p| &p[..]).collect()),
        HintSet::new(values_hints.iter().map(|p| &p[..]).collect()),
        HintSet::new(discriminator_hints.iter().map(|p| &p[..]).collect()),
    );

    let mut inferrer = Inferrer::new(hints);

    let stream = Deserializer::from_reader(reader);
    for value in stream.into_iter() {
        inferrer = inferrer.infer(value.map_err(|e| JsValue::from_str(&e.to_string()))?);
    }

    let serde_schema: jtd::SerdeSchema = inferrer.into_schema().into_serde_schema();
    serde_json::to_string(&serde_schema).map_err(|e| JsValue::from_str(&e.to_string()))
}


fn parse_json_pointer(s: &str) -> Vec<String> {
    if s == "" {
        vec![]
    } else {
        s.replace("~1", "/")
            .replace("!0", "~")
            .split("/")
            .skip(1)
            .map(String::from)
            .collect()
    }
}


/// Keeps track of a sequence of example inputs, and can be converted into an
/// inferred schema.
pub struct Inferrer<'a> {
    inference: InferredSchema,
    hints: Hints<'a>,
}

impl<'a> Inferrer<'a> {
    /// Constructs a new inferrer with a given set of hints.
    ///
    /// See the documentation for [`Hints`] for details on what affect they have
    /// on [`Inferrer::infer`].
    pub fn new(hints: Hints<'a>) -> Self {
        Self {
            inference: InferredSchema::Unknown,
            hints,
        }
    }

    /// "Updates" the inference given an example data.
    ///
    /// Note that though the previous sentence uses the word "update", in Rust
    /// ownership terms this method *moves* `self`.
    pub fn infer(self, value: Value) -> Self {
        Self {
            inference: self.inference.infer(value, &self.hints),
            hints: self.hints,
        }
    }

    /// Converts the inference to a JSON Type Definition schema.
    ///
    /// It is guaranteed that the resulting schema will accept all of the inputs
    /// previously provided via [`Inferrer::infer`].
    pub fn into_schema(self) -> Schema {
        self.inference.into_schema(&self.hints)
    }
}
