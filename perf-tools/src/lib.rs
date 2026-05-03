use std::arch::asm;
use std::time::Duration;

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

pub fn rdtsc_to_millis(frequency: f64, rdtsc: u64) -> f64 {
    rdtsc as f64 / frequency * 1000f64
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
