fn main() {
    let start = perf_tools::get_page_faults();
    // println!("start faults: {start}");
    let memory = vec![0u8; 100000];
    std::hint::black_box(&memory);
    let end = perf_tools::get_page_faults();
    println!("got page faults: {}", end - start);
    println!("expected page faults: {}", (memory.len() + (4096 - 1)) / 4096);
}
