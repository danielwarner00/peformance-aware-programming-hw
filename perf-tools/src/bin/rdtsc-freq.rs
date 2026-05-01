use perf_tools::rdtsc;
use std::time::Duration;

fn main() {
    let start = rdtsc();
    std::thread::sleep(Duration::from_secs(1));
    let end = rdtsc();
    println!(
        "rdtsc clock frequency: {:.2} GHz",
        (end - start) as f64 / 1e9
    );
}
