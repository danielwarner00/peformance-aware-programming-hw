use perf_tools::profile;

fn main() {
    println!("finished with {}", recurse(10_000));
}

fn recurse(remaining: u32) -> u32 {
    profile!("recurse");

    eprintln!("in");

    let mut result = 0;

    if remaining != 0 {
        result = remaining + recurse(remaining - 1);
    }

    eprintln!("out");
    return result;
}
