use std::arch::asm;

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
