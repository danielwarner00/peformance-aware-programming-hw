use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    match command.as_ref().map(|s| s.as_str()) {
        Some("decode") => {
            decode();
            ExitCode::SUCCESS
        }
        Some(_) => {
            eprintln!("no such command");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("no command provided");
            ExitCode::FAILURE
        }
    }
}

// reads instructions on stdin and output disassembled instructions to stdout
fn decode() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let mut instructions = input.as_slice();

    println!("bits 16");

    while !instructions.is_empty() {
        let instruction = instructions[0];
        let instruction_size = if instruction >> 2 == 0b100010 {
            // register/memory to/from register
            print_register_memory_instruction(instructions, "mov")
        } else if instruction >> 2 == 0
            || instruction >> 2 == 0b001010
            || instruction >> 2 == 0b001110
        {
            print_register_memory_instruction(
                instructions,
                three_bit_to_opcode(instruction >> 3 & 7),
            )
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
            assert_eq!(instructions[1] >> 3 & 7, 0);
            print_immediate_to_registry_memory_instruction(
                instructions,
                "mov",
                false,
                instruction & 1 == 1,
            )
        } else if instruction >> 2 == 0b100000 {
            let s = instruction >> 1 & 1 == 1;
            print_immediate_to_registry_memory_instruction(
                instructions,
                three_bit_to_opcode(instructions[1] >> 3 & 7),
                s,
                !s && instruction & 1 == 1,
            )
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
        } else if instruction >> 1 == 0b10
            || instruction >> 1 == 0b10110
            || instruction >> 1 == 0b11110
        {
            let code = instruction >> 1;
            let opcode = match code {
                0b10 => "add",
                0b10110 => "sub",
                0b11110 => "cmp",
                _ => unreachable!(),
            };
            print_immediate_to_accumulator_instruction(instructions, opcode)
        } else {
            print_simple_offset_instruction(
                instructions,
                match instruction {
                    0x74 => "je",
                    0x7c => "jl",
                    0x7e => "jle",
                    0x72 => "jb",
                    0x76 => "jbe",
                    0x7a => "jp",
                    0x70 => "jo",
                    0x78 => "js",
                    0x75 => "jne",
                    0x7d => "jnl",
                    0x7f => "jnle",
                    0x73 => "jnb",
                    0x77 => "jnbe",
                    0x7b => "jnp",
                    0x71 => "jno",
                    0x79 => "jns",
                    0xe2 => "loop",
                    0xe1 => "loopz",
                    0xe0 => "loopnz",
                    0xe3 => "jcxz",
                    _ => panic!(),
                },
            )
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

// returns number of bytes in instruction
fn print_register_memory_instruction(instructions: &[u8], opcode: &'static str) -> usize {
    let d: bool = instructions[0] >> 1 & 1 > 0;
    let w: bool = instructions[0] & 1 == 1;
    let reg: u8 = instructions[1] >> 3 & 0b111;

    let reg_str = register_name(reg, w);

    print!("{opcode} ");
    if d {
        print!("{reg_str}, ");
        print_register_memory(instructions);
        println!();
    } else {
        print_register_memory(instructions);
        println!(", {reg_str}");
    };

    2 + instruction_displacement_bytes(instructions)
}

fn print_immediate_to_registry_memory_instruction(
    instructions: &[u8],
    opcode: &'static str,
    sign_extend: bool,
    wide: bool,
) -> usize {
    let r#mod = instructions[1] >> 6 & 3;
    let w = instructions[0] & 1 != 0;

    let data_start_index = 2 + instruction_displacement_bytes(instructions);

    let (data, data_bytes) = match (sign_extend, wide) {
        (false, true) => (
            (instructions[data_start_index] as u16)
                | (instructions[data_start_index + 1] as u16) << 8,
            2,
        ),
        (true, true) => (instructions[data_start_index] as i8 as u16, 2),
        _ => (instructions[data_start_index] as u16, 1),
    };

    let memory = r#mod != 0b11;

    let destination_size = if memory {
        if w { "word " } else { "byte " }
    } else {
        ""
    };
    print!("{opcode} {destination_size}");
    print_register_memory(instructions);
    println!(", {data}");

    data_start_index + data_bytes
}

fn print_immediate_to_accumulator_instruction(instructions: &[u8], opcode: &'static str) -> usize {
    let w = instructions[0] & 1 == 1;

    let data = if w {
        instructions[1] as u16 | (instructions[2] as u16) << 8
    } else {
        instructions[1] as u16
    };

    let register = if w { "ax" } else { "al" };

    println!("{opcode} {register}, {data}");

    if w { 3 } else { 2 }
}

fn print_simple_offset_instruction(instructions: &[u8], opcode: &'static str) -> usize {
    let data = instructions[1] as i8;
    println!("{opcode}, {data}");
    2
}

fn three_bit_to_opcode(three_bit: u8) -> &'static str {
    assert!(three_bit < 8);
    match three_bit {
        0b000 => "add",
        0b101 => "sub",
        0b111 => "cmp",
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
