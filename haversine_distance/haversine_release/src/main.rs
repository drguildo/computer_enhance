mod json;

use json::Json;

use crate::json::parse_str;

use std::env;
use std::fs;
use std::io::Read;
use std::os::windows::fs::MetadataExt;

#[derive(Debug)]
struct Pair(f64, f64, f64, f64);

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: {} [haversine_input.json]", &args[0]);
        eprintln!("       {} [haversine_input.json] [answers.f64]", &args[0]);
        return;
    }
    let json_path = &args[1];

    let pairs = get_pairs_from_json(json_path);
    let pairs_count = pairs.len();

    let mut haversines: Vec<f64> = vec![];
    for pair in &pairs {
        let haversine = reference_haversine(pair.0, pair.1, pair.2, pair.3);
        haversines.push(haversine);
    }
    let sum: f64 = haversines.iter().sum();
    let avg = sum / haversines.len() as f64;

    println!(
        "Input size: {}",
        fs::metadata(json_path)
            .expect("Failed to read JSON file metadata")
            .file_size()
    );
    println!("Pair count: {}", pairs_count);
    println!("Haversine sum: {}", avg);

    if args.len() > 2 {
        let answer_path = &args[2];
        let mut answer_file = fs::File::open(answer_path).expect("Failed to open answers file");

        let mut buffer = [0; 8];
        let n = answer_file
            .read(&mut buffer)
            .expect("Failed to read answer");
        if n != 8 {
            panic!("Invalid number of bytes returned {} when reading answer", n);
        }
        let answer: f64 = f64::from_le_bytes(buffer);

        println!("");
        println!("Validation:");
        println!("Reference sum: {}", answer);
        println!("Difference: {}", avg - answer);
    }
}

fn get_pairs_from_json(json_path: &str) -> Vec<Pair> {
    let mut pairs: Vec<Pair> = vec![];

    let json_str = fs::read_to_string(json_path).unwrap();
    let json = parse_str(json_str.as_str()).expect("Failed to parse JSON");
    if let Json::Object(root) = json {
        if let Some(Json::Array(array)) = root.get("pairs") {
            for pair in array {
                if let Json::Object(pair_object) = pair {
                    let x0 = number_to_f64(pair_object.get("x0").expect("Failed to get x0 field"));
                    let y0 = number_to_f64(pair_object.get("y0").expect("Failed to get y0 field"));

                    let x1 = number_to_f64(pair_object.get("x1").expect("Failed to get x1 field"));
                    let y1 = number_to_f64(pair_object.get("y1").expect("Failed to get y1 field"));

                    pairs.push(Pair(x0, y0, x1, y1));
                }
            }
        }
    }

    pairs
}

fn number_to_f64(number: &Json) -> f64 {
    if let Json::Number {
        integer,
        fraction,
        precision,
        exponent: _,
    } = number
    {
        return format!("{}.{:0width$}", integer, fraction, width = precision)
            .parse()
            .unwrap();
    } else {
        panic!("Not a number type");
    }
}

fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    const EARTH_RADIUS_KILOMETER: f64 = 6371.0_f64;

    let dy = (y1 - y0).to_radians();
    let dx = (x1 - x0).to_radians();
    let y0 = y0.to_radians();
    let y1 = y1.to_radians();

    let root_term = (dy / 2.0).sin().powi(2) + y0.cos() * y1.cos() * (dx / 2.0).sin().powi(2);

    2.0 * EARTH_RADIUS_KILOMETER * root_term.sqrt().asin()
}
