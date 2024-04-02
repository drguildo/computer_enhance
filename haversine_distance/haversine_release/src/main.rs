mod json;

use json::Json;

use crate::json::parse_str;

use std::env;
use std::fs;

// use std::io::Read;
use std::os::windows::fs::MetadataExt;

#[derive(Debug)]
struct Pair(f64, f64, f64, f64);

// struct RefAnswers {
//     _answers: Vec<f64>,
//     sum: f64,
// }

struct PerfStats {
    init: u64,
    read: u64,
    parse: u64,
    sum: u64,
    end: u64,
}

fn main() {
    let mut perf_stats = PerfStats {
        init: 0,
        read: 0,
        parse: 0,
        sum: 0,
        end: 0,
    };

    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: {} [haversine_input.json]", &args[0]);
        eprintln!("       {} [haversine_input.json] [answers.f64]", &args[0]);
        return;
    }
    let json_path = &args[1];

    perf_stats.init = profile::read_cpu_timer();

    let json = fs::read_to_string(json_path).unwrap();
    perf_stats.read = profile::read_cpu_timer();

    let pairs = parse_json(&json);
    perf_stats.parse = profile::read_cpu_timer();

    let mut haversines: Vec<f64> = vec![];
    for pair in &pairs {
        let haversine = reference_haversine(pair.0, pair.1, pair.2, pair.3);
        haversines.push(haversine);
    }
    let sum: f64 = haversines.iter().sum();
    let avg = sum / haversines.len() as f64;
    perf_stats.sum = profile::read_cpu_timer();

    println!(
        "Input size: {}",
        fs::metadata(json_path)
            .expect("Failed to read JSON file metadata")
            .file_size()
    );
    println!("Pair count: {}", pairs.len());
    println!("Haversine sum: {}", avg);

    // if args.len() > 2 {
    //     let answers_path = &args[2];
    //     let answers = read_answers(answers_path);

    //     println!("");
    //     println!("Validation:");
    //     println!("Reference sum: {}", answers.sum);
    //     println!("Difference: {}", avg - answers.sum);
    // }
    perf_stats.end = profile::read_cpu_timer();

    fn percent(numerator: u64, denominator: u64) -> String {
        format!("{:.2}%", (numerator as f64 / denominator as f64) * 100_f64)
    }

    let cpu_freq = profile::estimate_cpu_timer_freq(None);
    let total_cpu_time_taken = perf_stats.end - perf_stats.init;
    println!(
        "\nTotal time: {}ms (CPU freq {})",
        total_cpu_time_taken / (cpu_freq / 1000),
        cpu_freq
    );
    println!(
        "  Read: {} ({})",
        perf_stats.read - perf_stats.init,
        percent(perf_stats.read - perf_stats.init, total_cpu_time_taken)
    );
    println!(
        "  Parse: {} ({})",
        perf_stats.parse - perf_stats.read,
        percent(perf_stats.parse - perf_stats.read, total_cpu_time_taken)
    );
    println!(
        "  Sum: {} ({})",
        perf_stats.sum - perf_stats.parse,
        percent(perf_stats.sum - perf_stats.parse, total_cpu_time_taken)
    );
}

fn parse_json(json: &str) -> Vec<Pair> {
    let mut pairs: Vec<Pair> = vec![];

    let json = parse_str(json).expect("Failed to parse JSON");
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

// fn read_answers(answers_path: &str) -> RefAnswers {
//     let answers_file = fs::File::open(answers_path).expect("Failed to open answers file");
//     let mut answers_reader = std::io::BufReader::new(answers_file);

//     let mut answers: Vec<f64> = vec![];

//     let mut buffer = [0; 8];
//     loop {
//         let num_bytes_read = answers_reader
//             .read(&mut buffer)
//             .expect("Failed to read answers");
//         if num_bytes_read == 0 {
//             break;
//         }
//         if num_bytes_read != buffer.len() {
//             panic!("Failed to fill answer buffer");
//         }
//         let answer: f64 = f64::from_le_bytes(buffer);
//         answers.push(answer);
//     }

//     if let Some(sum) = answers.pop() {
//         RefAnswers {
//             _answers: answers,
//             sum,
//         }
//     } else {
//         panic!("Answers file is empty")
//     }
// }

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
