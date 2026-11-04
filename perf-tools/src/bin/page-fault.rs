use perf_tools::RepetitionTester;
use std::io::Read;

const RESUSE_BUFFER: bool = false;

fn main() {
    let test_path = std::env::args().nth(1).expect("test file not provided");
    let timeout_seconds = 5f64;

    let file_len = std::fs::metadata(&test_path).unwrap().len();
    let mut input_buffer = vec![0; file_len.try_into().unwrap()];

    if RESUSE_BUFFER {
        let mut tester = RepetitionTester::new(timeout_seconds);

        while tester.continue_test() {
            let mut file = std::fs::File::open(&test_path).unwrap();

            tester.start();

            assert!(file.read(&mut input_buffer).unwrap() == usize::try_from(file_len).unwrap());

            tester.end();

            // file closes here
        }

        tester.print_stats("file read with reusing buffer");
    } else {
        let mut tester = RepetitionTester::new(timeout_seconds);

        while tester.continue_test() {
            let mut file = std::fs::File::open(&test_path).unwrap();
            let mut input_buffer = vec![0; file_len.try_into().unwrap()];

            tester.start();

            assert!(file.read(&mut input_buffer).unwrap() == usize::try_from(file_len).unwrap());

            tester.end();

            // file closes here
        }

        tester.print_stats("file read with reusing buffer");
    }
}
