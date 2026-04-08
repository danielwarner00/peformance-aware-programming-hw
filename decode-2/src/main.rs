use std::io::Read;

fn main() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let mut instructions = input.as_slice();

    println!("bits 16");

    while !instructions.is_empty() {
        let instruction = instructions[0];
        let instruction_size = if instruction >> 2 == 0b100010 {
            // register/memory to/from register
            let d: bool = instruction >> 1 & 1 > 0;
            let w: bool = instruction & 1 == 1;
            let rm: u8 = instructions[1] & 0b111;
            let reg: u8 = instructions[1] >> 3 & 0b111;
            let r#mod: u8 = instructions[1] >> 6 & 0b11;

            let reg_str = register_name(reg, w);

            let (base_rm_str, displacement, memory, displacement_bytes) = if r#mod == 0b11 {
                (register_name(rm, w), 0, false, 0)
            } else if r#mod == 0b00 && rm == 0b110 {
                (
                    "",
                    (instructions[2] as u16) | (instructions[3] as u16) << 8,
                    true,
                    2,
                )
            } else {
                let (displacement, displacement_bytes) = match r#mod {
                    0b00 => (0, 0),
                    0b01 => (instructions[2] as u16, 1),
                    0b10 => ((instructions[2] as u16) | (instructions[3] as u16) << 8, 2),
                    _ => unreachable!(),
                };
                (
                    [
                        "bx + si", "bx + di", "bp + si", "bp + di", "si", "di", "bp", "bx",
                    ][usize::from(rm)],
                    displacement,
                    true,
                    displacement_bytes,
                )
            };

            let print_rm = || {
                if memory {
                    if displacement > 0 {
                        if base_rm_str.is_empty() {
                            print!("[{displacement}]")
                        } else {
                            print!("[{base_rm_str} + {displacement}]")
                        }
                    } else {
                        print!("[{base_rm_str}]")
                    }
                } else {
                    print!("{base_rm_str}");
                }
            };

            print!("mov ");
            if d {
                print!("{reg_str}, ");
                print_rm();
                println!();
            } else {
                print_rm();
                println!(", {reg_str}");
            };

            2 + displacement_bytes
        } else if instruction >> 4 == 0b1011 {
            // immediate to register
            let w = instruction >> 3 & 1 == 1;
            let reg = instruction & 7;
            let reg_name = register_name(reg, w);
            let (data, displacement_bytes) = if w {
                ((instructions[1] as u16) | (instructions[2] as u16) << 8, 2)
            } else {
                (instructions[1] as u16, 1)
            };

            println!("mov {reg_name}, {data}");

            1 + displacement_bytes
        } else {
            panic!();
        };

        instructions = &instructions[instruction_size..];
    }
}

fn register_name(reg: u8, w: bool) -> &'static str {
    static REGISTER_NAMES: [[&str; 8]; 2] = [
        ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"],
        ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"],
    ];
    REGISTER_NAMES[w as usize][reg as usize]
}
