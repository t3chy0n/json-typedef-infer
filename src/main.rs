use anyhow::Error;
use clap::{crate_version, load_yaml, App, AppSettings};
use jtd_infer::{HintSet, Hints, Inferrer, NumType};
use serde_json::Deserializer;
use std::fs::File;
use std::io::stdin;
use std::io::BufReader;
use std::io::Read;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub fn generate_schema(
    input: &str,
    enum_hints: Vec<String>,
    values_hints: Vec<String>,
    discriminator_hints: Vec<String>,
    default_number_type: &str,
) -> Result<String, JsValue> {
    // Parse the inputs. This replaces what `clap` did in the CLI.

    let reader = BufReader::new(Cursor::new(input));

    let enum_hints: Vec<Vec<_>> = enum_hints
        .iter()
        .map(|hint| parse_json_pointer(hint))
        .collect();

    let values_hints: Vec<Vec<_>> = values_hints
        .iter()
        .map(|hint| parse_json_pointer(hint))
        .collect();

    let discriminator_hints: Vec<Vec<_>> = discriminator_hints
        .iter()
        .map(|hint| parse_json_pointer(hint))
        .collect();

    let default_num_type = match default_number_type {
        // ... match arms similar to your main.rs
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
