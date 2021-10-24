mod lib;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

fn test_serialization() {
    let mut json_value = lib::Value::Obj(HashMap::new());

    match json_value {
        lib::Value::Obj(ref mut obj_value) => {
            obj_value.insert(
                String::from("arrayField"),
                Box::new(lib::Value::Arr(vec![
                    Box::new(lib::Value::Str(String::from("abacaba"))),
                    Box::new(lib::Value::Bool(false)),
                    Box::new(lib::Value::Num(42.0)),
                ])),
            );

            obj_value.insert(String::from("nullField"), Box::new(lib::Value::Null));
        }
        _ => (),
    }

    println!("JSON value = {}", lib::serialize_value(&json_value));
}

fn test_deserialization() {
    let json_buffer = "{
            \"arrayField\": [
                \"abacaba\",
                false,
                42
            ],
            \"nullField\": null
        }";

    let maybe_json_value = lib::deserialize_value(json_buffer);
    match maybe_json_value {
        Some(json_value) => {
            println!(
                "Parsed JSON value (roundtrip) = {}",
                lib::serialize_value(&json_value)
            );
        }

        None => {
            println!("Failed to parse JSON value!");
        }
    }
}

fn test_deserialization_external() {
    let maybe_file_path = env::args().nth(1);
    if maybe_file_path.is_none() {
        println!("No file path is specified, can't run a test!");
        return;
    }

    let maybe_file = File::open(maybe_file_path.unwrap());
    if maybe_file.is_err() {
        println!("Failed to open the file!");
        return;
    }

    let mut file = maybe_file.unwrap();
    let mut file_data = String::new();
    if file.read_to_string(&mut file_data).is_err() {
        println!("Failed to read the file!");
        return;
    }

    println!("Starting...");

    let start = Instant::now();

    for _ in 0..1000 {
        let maybe_json_value = lib::deserialize_value(&file_data);
        if maybe_json_value.is_none() {
            println!("Failed to deserialize the file as JSON!");
            return;
        }
    }

    let end = start.elapsed();
    println!("Deserialization: {} us", end.as_micros());
    println!("Deserialization: {} s", end.as_secs());
}

fn main() {
    test_serialization();
    test_deserialization();
    test_deserialization_external();
}
