use perf_tools::gettime;

fn main() {
    let monotonic = gettime(libc::CLOCK_MONOTONIC).unwrap();
    println!("got monotonic clock: {}ns", monotonic.as_nanos());
}
