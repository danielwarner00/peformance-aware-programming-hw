use clap::Parser;
use rand;

#[derive(Parser)]
struct Args {
    count: u32,
    // TODO add
    // #[arg(long, default_value_t = 0)]
    // seed: i32,
}

fn main() {
    let args = Args::parse();

    print!("{{\"pairs\":[");
    let mut first = true;
    let mut haversine_sum = 0.0;
    for _ in 0..args.count {
        fn generate_lat() -> f64 {
            (rand::random::<f64>() * 2.0 - 1.0).asin().to_degrees() // the argument to asin will never be NaN
        }
        fn generate_long() -> f64 {
            (rand::random::<f64>() - 0.5) * 360.0
        }
        let lat0 = generate_lat();
        let long0 = generate_long();
        let lat1 = generate_lat();
        let long1 = generate_long();

        haversine_sum += haversine_of_degrees(lat0, long0, lat1, long1, 6371.0);

        if !first {
            print!(",");
        }
        print!(r#"{{"lat0":{lat0},"long0":{long0},"lat1":{lat1},"long1":{long1}}}"#);

        first = false;
    }
    let average = {haversine_sum / args.count as f64 };
    // also output the sum and average for verifying computations
    print!(r#"],"sum":{haversine_sum},"average":{average}}}"#);
}

// baseline naïve version given in the course
fn haversine_of_degrees(lat0: f64, long0: f64, lat1: f64, long1: f64, radius: f64) -> f64 {
    let d_y = (lat1 - lat0).to_radians();
    let d_x = (long1 - long0).to_radians();
    let y0 = lat0.to_radians();
    let y1 = lat1.to_radians();

    let root_term = (d_y / 2.0).sin().powi(2) + y0.cos() * y1.cos() * (d_x / 2.0).sin().powi(2);
    2.0 * radius * root_term.sqrt().asin()
}
