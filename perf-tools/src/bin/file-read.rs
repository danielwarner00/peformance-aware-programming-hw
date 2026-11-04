use perf_tools::RepetitionTester;
use std::io::Read;

fn main() {
    let test_path = std::env::args().nth(1).expect("test file not provided");
    let timeout_seconds = 5f64;

    {
        let mut tester = RepetitionTester::new(timeout_seconds);

        while tester.continue_test() {
            tester.start();
            std::fs::read(&test_path).unwrap();
            tester.end();
        }

        tester.print_stats("std::fs::read");
    }

    {
        let mut tester = RepetitionTester::new(timeout_seconds);

        while tester.continue_test() {
            tester.start();

            let mut input_buffer = Vec::<u8>::new();
            let mut file = std::fs::File::open(&test_path).unwrap();
            file.read_to_end(&mut input_buffer).unwrap();

            tester.end();
        }

        tester.print_stats("<std::fs::File as Read>::read_to_end with declared buffer");
    }

    {
        let mut tester = RepetitionTester::new(timeout_seconds);

        while tester.continue_test() {
            tester.start();
            std::fs::read(&test_path).unwrap();
            tester.end();
        }

        tester.print_stats("std::fs::read");
    }
}
