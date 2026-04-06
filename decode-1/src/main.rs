use std::io::Read;

fn main() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let input = input.as_slice();

    let (instructions, remainder) = input.as_chunks::<2>();
    assert!(remainder.is_empty());

    println!("bits 16");

    for instruction in instructions {
        let instruction: u16 = instruction[0] as u16 | ((instruction[1] as u16) << 8);

        assert!(instruction & 0xfc == 0b100010 << 2);
        let w: u8 = (instruction & 0b1).try_into().unwrap();
        let d: bool = instruction >> 1 & 0b1 > 0;
        let rm: u8 = (instruction >> 8 & 0b111).try_into().unwrap();
        let reg: u8 = (instruction >> 11 & 0b111).try_into().unwrap();
        let r#mod: u8 = (instruction >> 14 & 0b11).try_into().unwrap();

        assert!(r#mod == 0b11);

        let (destination, source) = if d { (reg, rm) } else { (rm, reg) };

        static REGISTER_NAMES: [[&'static str; 8]; 2] = [
            ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"],
            ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"],
        ];

        let names: &[&'static str; 8] = &REGISTER_NAMES[usize::from(w)];
        println!(
            "mov {}, {}",
            names[usize::from(destination)],
            names[usize::from(source)]
        );
    }
}
