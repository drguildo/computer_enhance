use std::{env::args, io::Write, process::ExitCode};

struct Point {
    x: f64,
    y: f64,
}

fn main() -> ExitCode {
    let args: Vec<String> = args().collect();

    if args.len() < 4 {
        eprintln!(
            "Usage: {} [uniform/cluster] [random seed] [number of coordinate pairs to generate]",
            args[0]
        );
        return ExitCode::FAILURE;
    }

    let method = &args[1];
    let random_seed: u64 = args[2].parse().expect("Failed to parse random seed");
    let pair_count: u64 = args[3]
        .parse()
        .expect("Failed to parse number of coordinate pairs");

    fastrand::seed(random_seed);

    let mut haversines: Vec<f64> = vec![];

    let json_filename = format!("data_{}_flex.json", pair_count);
    let json_file = std::fs::File::create(json_filename).expect("Unable to create JSON file");
    let mut json_writer = std::io::BufWriter::new(json_file);

    let answers_filename = format!("data_{}_haveranswer.f64", pair_count);
    let answers_file =
        std::fs::File::create(answers_filename).expect("Unable to create answers file");
    let mut answers_writer = std::io::BufWriter::new(answers_file);

    let coords = if method == "uniform" {
        generate_coordinates_uniform(pair_count)
    } else {
        let clusters = generate_clusters(64);
        generate_coordinates_cluster(pair_count, &clusters)
    };

    json_writer
        .write_all("{\"pairs\":[\n".as_bytes())
        .expect("Failed to write to file");
    for (i, coord) in coords.iter().enumerate() {
        let haversine = reference_haversine(coord.0.x, coord.0.y, coord.1.x, coord.1.y);
        haversines.push(haversine);
        answers_writer
            .write(&haversine.to_le_bytes())
            .expect("Failed to write to answers file");

        json_writer
            .write_all(
                format!(
                    "    {{\"x0\":{}, \"y0\":{}, \"x1\":{}, \"y1\":{}}}",
                    coord.0.x, coord.0.y, coord.1.x, coord.1.y
                )
                .as_bytes(),
            )
            .expect("Failed to write to file");

        if i == coords.len() - 1 {
            json_writer
                .write_all("\n".as_bytes())
                .expect("Failed to write to file");
        } else {
            json_writer
                .write_all(",\n".as_bytes())
                .expect("Failed to write to file");
        }
    }
    json_writer
        .write_all("]}".as_bytes())
        .expect("Failed to write to file");

    let sum: f64 = haversines.iter().sum();
    let avg = sum / haversines.len() as f64;

    answers_writer
        .write(&avg.to_le_bytes())
        .expect("Failed to write average sum");

    println!("Method: {}", method);
    println!("Random seed: {}", random_seed);
    println!("Pair count: {}", pair_count);
    println!("Expected sum: {}", avg);

    ExitCode::SUCCESS
}

fn generate_coordinates_uniform(num_coords: u64) -> Vec<(Point, Point)> {
    let mut coords = vec![];

    for _ in 0..num_coords {
        let x0 = (fastrand::f64() * 360.0) - 180.0;
        let y0 = (fastrand::f64() * 180.0) - 90.0;

        debug_assert!(x0 >= -180.0 && x0 <= 180.0);
        debug_assert!(y0 >= -90.0 && y0 <= 90.0);

        let x1 = (fastrand::f64() * 360.0) - 180.0;
        let y1 = (fastrand::f64() * 180.0) - 90.0;

        debug_assert!(x1 >= -180.0 && x1 <= 180.0);
        debug_assert!(y1 >= -90.0 && y1 <= 90.0);

        coords.push((Point { x: x0, y: y0 }, Point { x: x1, y: y1 }));
    }

    coords
}

fn generate_coordinates_cluster(num_coords: u64, clusters: &[Point]) -> Vec<(Point, Point)> {
    let mut coords = vec![];

    let mut cluster: &Point;
    for _ in 0..num_coords {
        cluster = fastrand::choice(clusters).unwrap();
        let x0 = cluster.x + fastrand::f64() * 10.0;
        let y0 = cluster.y + fastrand::f64() * 10.0;

        debug_assert!(x0 >= -180.0 && x0 <= 180.0);
        debug_assert!(y0 >= -90.0 && y0 <= 90.0);

        cluster = fastrand::choice(clusters).unwrap();
        let x1 = cluster.x + fastrand::f64() * 10.0;
        let y1 = cluster.y + fastrand::f64() * 10.0;

        debug_assert!(x1 >= -180.0 && x1 <= 180.0);
        debug_assert!(y1 >= -90.0 && y1 <= 90.0);

        coords.push((Point { x: x0, y: y0 }, Point { x: x1, y: y1 }));
    }

    coords
}

fn generate_clusters(num_clusters: u8) -> Vec<Point> {
    let mut clusters = vec![];

    for _ in 0..num_clusters {
        // Generate a number in the range [-180, 170]
        let x = -180.0 + (350.0 * fastrand::f64());
        // Generate a number in the range [-90, 80]
        let y = -90.0 + (170.0 * fastrand::f64());

        debug_assert!(x >= -180.0 && x <= 170.0);
        debug_assert!(y >= -90.0 && y <= 80.0);

        clusters.push(Point { x, y });
    }

    clusters
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
