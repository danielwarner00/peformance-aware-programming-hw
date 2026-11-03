#[cfg(feature = "profile")]
use dtor::dtor;
// currently, users of this crate must have linkme as a direct dependency, TODO figure out how to
// remove that requirement
pub use linkme::distributed_slice;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

// this would have to be updated to work for others cpus but this works for now
const RDTSC_FREQUENCY: f64 = 4.7e9;

pub fn rdtsc() -> u64 {
    unsafe { std::arch::x86_64::_rdtsc() }
}

pub fn rdtsc_to_millis(rdtsc: u64) -> f64 {
    rdtsc as f64 / RDTSC_FREQUENCY * 1000f64
}

// TODO this only handles positive times, handle
fn timespec_to_duration(timespec: libc::timespec) -> Duration {
    Duration::new(timespec.tv_sec as u64, timespec.tv_nsec as u32)
}

pub fn gettime(clockid: libc::clockid_t) -> Result<Duration, ()> {
    let mut timespec = Default::default();
    let result;
    unsafe {
        result = libc::clock_gettime(clockid, &mut timespec);
    }
    if result == -1 {
        Err(())
    } else {
        Ok(timespec_to_duration(timespec))
    }
}

#[cfg(feature = "profile")]
struct Rdtsc(u64);

#[cfg(feature = "profile")]
#[test]
fn test_to_significant_figures() {
    assert_eq!(to_significant_figures(1234f64), 1230f64);
    assert_eq!(to_significant_figures(1.236f64), 1.24f64);
    assert_eq!(to_significant_figures(0f64), 0f64);
}

// rounds to three significant figures
#[cfg(feature = "profile")]
fn to_significant_figures(value: f64) -> f64 {
    let mut power = value.log10();
    if power == f64::NEG_INFINITY {
        return 0f64;
    } else {
        power = power.floor();
    }
    let divisor = 10f64.powf(power - 2f64);
    (value / divisor).round() * divisor
}

#[cfg(feature = "profile")]
impl std::fmt::Display for Rdtsc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let milliseconds = (self.0 as f64 / RDTSC_FREQUENCY) * 1000f64;
        let number = to_significant_figures(milliseconds);
        let floor = number.log10().floor();
        let decimal_places = if floor == f64::NEG_INFINITY {
            1
        } else {
            std::cmp::min(2, (floor as i32 - 2) as usize)
        };
        write!(f, "{:.decimal_places$} ms", to_significant_figures(number))
    }
}

pub struct Counter {
    pub root_counter: AtomicU64,
    pub leaf_counter: AtomicU64,
    pub name: &'static str,
    pub in_counter: AtomicBool, // TODO this field is why the current implementation doesn't quite
    // work with multiple threads; really this field should be a thread local or I need to do
    // something else to determine whether a thread is alredy in a counter when pushing it to the
    // stack
    pub hits: AtomicU64,
    pub data_size: AtomicU64, // amount of data processed by this counter
}

struct CounterStack {
    stack: [Option<CounterStackEntry>; 4096],
    index: usize,
}

#[derive(Clone, Copy)]
struct CounterStackEntry {
    counter: &'static Counter,
    leaf_start: u64, // time that this entry most recently became the leaf, either because this
    // entry started or the previous leaf ended
    root_start: Option<u64>, // only Some if the this entry is the first entry of this Counter on the stack, in which case the value is the time that it started
}

impl CounterStack {
    fn push(&mut self, entry: CounterStackEntry) {
        self.stack[self.index] = Some(entry);

        self.index += 1;
    }

    fn pop(&mut self) -> Option<CounterStackEntry> {
        if self.index == 0 {
            None
        } else {
            self.index -= 1;
            Some(self.stack[self.index].unwrap())
        }
    }

    fn peek(&self) -> Option<&CounterStackEntry> {
        if self.index == 0 {
            None
        } else {
            self.stack[self.index - 1].as_ref()
        }
    }

    fn peek_mut(&mut self) -> Option<&mut CounterStackEntry> {
        if self.index == 0 {
            None
        } else {
            self.stack[self.index - 1].as_mut()
        }
    }
}

#[distributed_slice]
pub static COUNTERS: [Counter];

thread_local! {
    static COUNTER_STACK: RefCell<CounterStack> = const { RefCell::new(CounterStack {
        stack: [None; _],
        index: 0,
    })};
}

// start a new counter, replacing the current counter as the leaf
pub fn start_counter(counter: &'static Counter) {
    let now = rdtsc();

    COUNTER_STACK.with_borrow(|counter_stack| {
        if let Some(CounterStackEntry {
            counter: current_counter,
            leaf_start,
            ..
        }) = counter_stack.peek()
        {
            current_counter
                .leaf_counter
                .fetch_add(now - leaf_start, Ordering::Relaxed);
        }
    });

    push_counter(CounterStackEntry {
        counter,
        leaf_start: now,
        root_start: if counter.in_counter.load(Ordering::Relaxed) {
            None
        } else {
            Some(now)
        },
    });
    counter.in_counter.store(true, Ordering::Relaxed);
    counter.hits.fetch_add(1, Ordering::Relaxed);
}

// end the current counter, making the previous counter the leaf
fn end_counter() {
    let now = rdtsc();

    let CounterStackEntry {
        counter: current_counter,
        leaf_start,
        root_start,
    } = pop_counter().expect("no current counter");
    let duration = now - leaf_start;
    current_counter
        .leaf_counter
        .fetch_add(duration, Ordering::Relaxed);
    if let Some(root_start) = root_start {
        let duration = now - root_start;
        current_counter
            .root_counter
            .fetch_add(duration, Ordering::Relaxed);
        current_counter.in_counter.store(false, Ordering::Relaxed);
    }

    COUNTER_STACK.with_borrow_mut(|counter_stack| {
        if let Some(entry) = counter_stack.peek_mut() {
            entry.leaf_start = now;
        }
    })
}

fn push_counter(entry: CounterStackEntry) {
    COUNTER_STACK.with_borrow_mut(|stack| stack.push(entry));
}

fn pop_counter() -> Option<CounterStackEntry> {
    COUNTER_STACK.with_borrow_mut(|stack| stack.pop())
}

pub struct ProfilingSection;

impl Drop for ProfilingSection {
    fn drop(&mut self) {
        end_counter();
    }
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! profile {
    ($name:expr) => {
        profile!($name, 0u64)
    };
    ($name:expr, $value:expr) => {
        let _ = {
            #[$crate::distributed_slice($crate::COUNTERS)]
            static COUNTER: $crate::Counter = const {
                $crate::Counter {
                    root_counter: std::sync::atomic::AtomicU64::new(0),
                    leaf_counter: std::sync::atomic::AtomicU64::new(0),
                    name: $name,
                    in_counter: std::sync::atomic::AtomicBool::new(false),
                    hits: std::sync::atomic::AtomicU64::new(0),
                    data_size: std::sync::atomic::AtomicU64::new(0),
                }
            };

            COUNTER.data_size.fetch_add($value as u64, std::sync::atomic::Ordering::Relaxed); // TODO
            // do something other than as u64 here
            // TODO confirm this gets optimized out when $value is 0 so that profile! uses that don't
            // use data_size aren't slowed down more than they need to be
            $crate::start_counter(&COUNTER);
            &$crate::ProfilingSection
        };
    };
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! profile {
    ($name:expr) => {};
    ($name:expr, $value:expr) => {}
}

#[macro_export]
macro_rules! declare_counters {
    () => {
        #[$crate::gather]
        static COUNTERS: ScatteredSlice<&'static Counter>;
    };
}

#[test]
fn test_profile() {
    profile!("hello");
}

#[cfg(feature = "profile")]
fn print_counters() {
    for Counter {
        name,
        leaf_counter,
        root_counter,
        hits,
        data_size,
        ..
    } in &*COUNTERS
    {
        let leaf_time = Rdtsc(leaf_counter.load(Ordering::Relaxed));
        let root_time_count = root_counter.load(Ordering::Relaxed);
        let root_time = Rdtsc(root_time_count);
        let hits = hits.load(Ordering::Relaxed);
        let data_size = data_size.load(Ordering::Relaxed);
        let s = if hits == 1 { "" } else { "s" };
        print!(
            "For counter \"{name}\" with {hits} hit{s} got root time {root_time}, leaf time {leaf_time}"
        );
        assert!(root_time_count <= 1 << 53);
        let root_time_seconds = root_time_count as f64 / RDTSC_FREQUENCY;
        assert!(data_size <= 1 << 53);
        let throughput = data_size as f64 / root_time_seconds / 1e9;
        let data_size_mb = data_size as f64 / 1e6;
        if data_size != 0 {
            print!(", procesed {data_size_mb:.3} MB at {throughput:.3} GB/s");
        }
        println!();
    }
}

#[cfg(feature = "profile")]
#[dtor(unsafe)]
fn dtor() {
    print_counters();
}

/*
#[macro_export]
macro_rules! _profile_block_impl {
    ($name:expr, $b:block) => {
        {
            thread_local! {
                static _COUNTER: $crate::Counter = const {
                    $crate::Counter {
                        counter: std::cell::Cell::new(0),
                        name: $name
                    }
                };
            }
            let start = $crate::rdtsc();
            let result = $b;
            let duration = $crate::rdtsc() - start;
            _COUNTER.with(|counter| {
                counter.counter.set(counter.counter.get() + duration);
            });
            result
        }
    }
}

#[macro_export]
macro_rules! profile_block {
    ($name:expr, $b:block) => {
        $crate::_profile_block_impl!($name, $b)
    };
    ($($b:tt)*) => {
        $crate::_profile_block_impl!(concat!(file!(), ":", line!()), { $($b)* })
    };
}

#[test]
fn test_profile_block() {
    let x = profile_block! {
        // let alt = _start;
        println!();
        10
    };
    assert_eq!(10, x);
}
*/
