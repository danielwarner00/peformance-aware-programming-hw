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
            let reg: u8 = instructions[1] >> 3 & 0b111;

            let reg_str = register_name(reg, w);

            print!("mov ");
            if d {
                print!("{reg_str}, ");
                print_register_memory(instructions);
                println!();
            } else {
                print_register_memory(instructions);
                println!(", {reg_str}");
            };

            2 + instruction_displacement_bytes(instructions)
        } else if instruction >> 4 == 0b1011 {
            // immediate to register
            let w = instruction >> 3 & 1 == 1;
            let reg = instruction & 7;
            let reg_name = register_name(reg, w);
            let (data, data_bytes) = if w {
                ((instructions[1] as u16) | (instructions[2] as u16) << 8, 2)
            } else {
                (instructions[1] as u16, 1)
            };

            println!("mov {reg_name}, {data}");

            1 + data_bytes
        } else if instruction >> 1 == 0b1100011 {
            // immediate to register/memory
            assert_eq!(instructions[1] >> 3 & 7, 0);
            let w = instruction & 1 == 1;
            let r#mod = instructions[1] >> 6 & 3;

            let data_start_index = 2 + instruction_displacement_bytes(instructions);

            let (data, data_bytes) = if w {
                (
                    (instructions[data_start_index] as u16)
                        | (instructions[data_start_index + 1] as u16) << 8,
                    2,
                )
            } else {
                (instructions[data_start_index] as u16, 1)
            };

            let memory = r#mod != 0b11;

            print!("mov ");
            print_register_memory(instructions);
            let size = if memory {
                if w { "word " } else { "byte " }
            } else {
                ""
            };
            println!(", {size}{data}");

            data_start_index + data_bytes
        } else if instruction >> 2 == 0b101000 {
            let w = instruction & 1 == 1;
            let d = instruction >> 1 & 1 == 1;

            let address = if w {
                instructions[1] as u16 | (instructions[2] as u16) << 8
            } else {
                instructions[1] as u16
            };

            print!("mov ");
            if d {
                println!("[{address}], ax");
            } else {
                println!("ax, [{address}]");
            }

            if w { 3 } else { 2 }
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

fn make_displacement(bytes: &[u8]) -> i16 {
    match bytes {
        [first] => *first as i8 as i16,
        [first, second] => (*first as u16 | (*second as u16) << 8) as i16,
        _ => panic!(),
    }
}

fn print_register_memory(instructions: &[u8]) {
    let w: bool = instructions[0] & 1 == 1;
    let rm: u8 = instructions[1] & 0b111;
    let r#mod: u8 = instructions[1] >> 6 & 0b11;

    let (base_rm_str, displacement, memory) = if r#mod == 0b11 {
        (register_name(rm, w), 0, false)
    } else if r#mod == 0b00 && rm == 0b110 {
        ("", make_displacement(&instructions[2..4]), true)
    } else {
        let displacement = match r#mod {
            0b00 => 0,
            0b01 => make_displacement(&instructions[2..3]),
            0b10 => make_displacement(&instructions[2..4]),
            _ => unreachable!(),
        };
        (
            [
                "bx + si", "bx + di", "bp + si", "bp + di", "si", "di", "bp", "bx",
            ][usize::from(rm)],
            displacement,
            true,
        )
    };

    if memory {
        if base_rm_str.is_empty() {
            print!("[{displacement}]")
        } else if displacement == 0 {
            print!("[{base_rm_str}]")
        } else {
            print!("[{base_rm_str} {displacement:+}]")
        }
    } else {
        print!("{base_rm_str}");
    }
}

fn instruction_displacement_bytes(instructions: &[u8]) -> usize {
    let rm: u8 = instructions[1] & 0b111;
    let r#mod: u8 = instructions[1] >> 6 & 0b11;

    match r#mod {
        0b00 => {
            if rm == 0b110 {
                2
            } else {
                0
            }
        }
        0b01 => 1,
        0b10 => 2,
        0b11 => 0,
        _ => unreachable!(),
    }
}
