use std::arch::asm;
use std::cell::Cell;
use std::thread::LocalKey;
use std::time::Duration;

// this would have to be updated to work for others cpus but this works for now
const RDTSC_FREQUENCY: f64 = 4.7e9;

pub fn rdtsc() -> u64 {
    let lower: u32;
    let upper: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") lower,
            out("edx") upper,
            options(nomem, preserves_flags),
        );
    }

    (upper as u64) << 32 | lower as u64
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

struct Rdtsc(u64);

#[test]
fn test_to_significant_figures() {
    assert_eq!(to_significant_figures(1234f64), 1230f64);
    assert_eq!(to_significant_figures(1.236f64), 1.24f64);
    assert_eq!(to_significant_figures(0f64), 0f64);
}

// rounds to three significant figures
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

impl std::fmt::Display for Rdtsc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let seconds = self.0 as f64 / RDTSC_FREQUENCY;
        let (mut number, unit) = if seconds < 1e-6f64 {
            (seconds * 1e9f64, "ns")
        } else if seconds < 1e-3 {
            (seconds * 1e6f64, "μs")
        } else if seconds < 1f64 {
            (seconds * 1000f64, "ms")
        } else {
            (seconds, "s")
        };
        number = to_significant_figures(number);
        let floor = number.log10().floor();
        let decimal_places = if floor == f64::NEG_INFINITY {
            1
        } else {
            std::cmp::min(2, (floor as i32 - 2) as usize)
        };
        write!(
            f,
            "{:.decimal_places$} {unit}",
            to_significant_figures(number)
        )
    }
}

pub struct Counter {
    pub counter: Cell<u64>,
    pub name: &'static str,
    pub in_section: Cell<bool>,
}

impl Drop for Counter {
    fn drop(&mut self) {
        eprint!("got {} time: {}", self.name, Rdtsc(self.counter.get()));
    }
}

pub struct ProfilingSection {
    start: u64,
    counter: &'static LocalKey<Counter>, // TODO figure out how I can remove this reference, check
    ignoring: bool,
}

impl ProfilingSection {
    pub fn new(counter: &'static LocalKey<Counter>, ignoring: bool) -> ProfilingSection {
        ProfilingSection {
            start: rdtsc(),
            counter,
            ignoring,
        }
    }
}

impl Drop for ProfilingSection {
    fn drop(&mut self) {
        if !self.ignoring {
            let duration = rdtsc() - self.start;
            self.counter.with(|counter| {
                counter.counter.set(counter.counter.get() + duration);
                counter.in_section.set(false);
            });
        }
    }
}

#[macro_export]
macro_rules! profile {
    ($name:expr) => {
        let _ = {
            thread_local! {
                static COUNTER: $crate::Counter = const { $crate::Counter { counter: std::cell::Cell::new(0), name: $name, in_section: std::cell::Cell::new(false) } };
            }
            let mut ignoring = false;
            COUNTER.with(|counter| {
                ignoring = counter.in_section.get();
                counter.in_section.set(true);
            });
            &$crate::ProfilingSection::new(&COUNTER, ignoring)
        };
    }
}

#[test]
fn test_profile() {
    profile!("hello");
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
