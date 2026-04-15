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

        if !first {
            print!(",");
        }
        print!(r#"{{"lat0":{lat0},"long0":{long0},"lat1":{lat1},"long1":{long1}}}"#);

        first = false;
    }
    print!("]}}");

    // TODO compute correct value and output
}
