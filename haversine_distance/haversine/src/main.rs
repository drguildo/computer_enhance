use std::time::Instant;

fn haversine_of_degrees(x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    const EARTH_RADIUS_KILOMETER: f64 = 6371.0_f64;

    let dy = (y1 - y0).to_radians();
    let dx = (x1 - x0).to_radians();
    let y0 = y0.to_radians();
    let y1 = y1.to_radians();

    let root_term = (dy / 2.0).sin().powi(2) + y0.cos() * y1.cos() * (dx / 2.0).sin().powi(2);

    2.0 * EARTH_RADIUS_KILOMETER * root_term.sqrt().asin()
}

fn main() {
    let json_string =
        std::fs::read_to_string("data_10000000_flex.json").expect("Failed to read JSON data");

    let start_time = Instant::now();
    let json_data: serde_json::Value =
        serde_json::from_str(&json_string).expect("Failed to parse JSON data");
    let mid_time = Instant::now();

    let mut sum = 0_f64;
    let mut count = 0_f64;

    let pairs = json_data["pairs"]
        .as_array()
        .expect("Failed to find JSON pairs attribute");
    for pair in pairs {
        let x0 = pair["x0"]
            .as_f64()
            .expect("Failed to find JSON x0 attribute");
        let y0 = pair["y0"]
            .as_f64()
            .expect("Failed to find JSON y0 attribute");
        let x1 = pair["x1"]
            .as_f64()
            .expect("Failed to find JSON x1 attribute");
        let y1 = pair["y1"]
            .as_f64()
            .expect("Failed to find JSON y1 attribute");
        sum = sum + haversine_of_degrees(x0, y0, x1, y1);
        count = count + 1_f64;
    }
    let average = sum / count;
    let end_time = Instant::now();

    println!("Result: {}", average);
    println!("Input = {} seconds", (mid_time - start_time).as_secs());
    println!("Math = {} seconds", (end_time - mid_time).as_secs());
    println!("Total = {} seconds", (end_time - start_time).as_secs());
    println!(
        "Throughput = {} haversines/second",
        count / (end_time - start_time).as_secs() as f64
    );
}
