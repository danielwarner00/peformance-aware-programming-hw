use std::fmt;
use std::fmt::Display;
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    match command.as_deref() {
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
        let (instruction, rest) = decode_instruction(instructions);
        println!("{}", instruction);
        instructions = rest
    }
}

#[derive(Clone, Copy)]
enum DisplacementBase {
    Bxsi,
    Bxdi,
    Bpsi,
    Bpdi,
    Si,
    Di,
    Bp,
    Bx,
}

impl Display for DisplacementBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DisplacementBase::Bxsi => "bx + si",
                DisplacementBase::Bxdi => "bx + di",
                DisplacementBase::Bpsi => "bp + si",
                DisplacementBase::Bpdi => "bp + di",
                DisplacementBase::Si => "si",
                DisplacementBase::Di => "di",
                DisplacementBase::Bp => "bp",
                DisplacementBase::Bx => "bx",
            }
        )
    }
}

impl DisplacementBase {
    fn from_rm_field(rm: u8) -> Self {
        [
            DisplacementBase::Bxsi,
            DisplacementBase::Bxdi,
            DisplacementBase::Bpsi,
            DisplacementBase::Bpdi,
            DisplacementBase::Si,
            DisplacementBase::Di,
            DisplacementBase::Bp,
            DisplacementBase::Bx,
        ][rm as usize]
    }
}

#[derive(Clone, Copy)]
enum MemoryLocation {
    Displaced {
        base: DisplacementBase,
        displacement: i16,
    },
    Direct(u16),
}

impl Display for MemoryLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryLocation::Displaced { base, displacement } => {
                write!(f, "[{} {:+}]", base, displacement)
            }
            MemoryLocation::Direct(value) => write!(f, "[{}]", value),
        }
    }
}
#[derive(Clone, Copy)]
enum Register {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl Register {
    fn from_reg_field(reg: u8, wide: bool) -> Self {
        (if wide {
            [
                Register::AX,
                Register::CX,
                Register::DX,
                Register::BX,
                Register::SP,
                Register::BP,
                Register::SI,
                Register::DI,
            ]
        } else {
            [
                Register::AL,
                Register::CL,
                Register::DL,
                Register::BL,
                Register::AH,
                Register::CH,
                Register::DH,
                Register::BH,
            ]
        })[reg as usize]
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Register::AL => "al",
                Register::CL => "cl",
                Register::DL => "dl",
                Register::BL => "bl",
                Register::AH => "ah",
                Register::CH => "ch",
                Register::DH => "dh",
                Register::BH => "bh",
                Register::AX => "ax",
                Register::CX => "cx",
                Register::DX => "dx",
                Register::BX => "bx",
                Register::SP => "sp",
                Register::BP => "bp",
                Register::SI => "si",
                Register::DI => "di",
            }
        )
    }
}

#[derive(Clone, Copy)]
enum Location {
    Register(Register),
    Memory(MemoryLocation),
}

impl Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Register(s) => write!(f, "{}", s),
            Location::Memory(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Copy)]
enum Operand8 {
    Immediate(u8),
    Location(Location),
}

impl Display for Operand8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand8::Immediate(s) => write!(f, "byte {}", s),
            Operand8::Location(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Copy)]
enum Operand16 {
    Immediate(u16),
    Location(Location),
}

impl Display for Operand16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand16::Immediate(s) => write!(f, "word {}", s),
            Operand16::Location(s) => write!(f, "{}", s),
        }
    }
}

// instructions
#[derive(Clone, Copy)]
enum BinaryOperation {
    Mov,
    Add,
    Sub,
    Cmp,
}

impl BinaryOperation {
    fn from_three_bit(three_bit: u8) -> BinaryOperation {
        assert!(three_bit < 8);
        match three_bit {
            0b000 => BinaryOperation::Add,
            0b101 => BinaryOperation::Sub,
            0b111 => BinaryOperation::Cmp,
            _ => panic!(),
        }
    }
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinaryOperation::Mov => "mov",
                BinaryOperation::Add => "add",
                BinaryOperation::Sub => "sub",
                BinaryOperation::Cmp => "cmp",
            }
        )
    }
}

#[derive(Clone, Copy)]
enum JumpOperation {
    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jne,
    Jnl,
    Jnle,
    Jnb,
    Jnbe,
    Jnp,
    Jno,
    Jns,
    Loop,
    Loopz,
    Loopnz,
    Jcxz,
}

impl JumpOperation {
    fn from_byte(b: u8) -> Option<JumpOperation> {
        match b {
            0x74 => Some(JumpOperation::Je),
            0x7c => Some(JumpOperation::Jl),
            0x7e => Some(JumpOperation::Jle),
            0x72 => Some(JumpOperation::Jb),
            0x76 => Some(JumpOperation::Jbe),
            0x7a => Some(JumpOperation::Jp),
            0x70 => Some(JumpOperation::Jo),
            0x78 => Some(JumpOperation::Js),
            0x75 => Some(JumpOperation::Jne),
            0x7d => Some(JumpOperation::Jnl),
            0x7f => Some(JumpOperation::Jnle),
            0x73 => Some(JumpOperation::Jnb),
            0x77 => Some(JumpOperation::Jnbe),
            0x7b => Some(JumpOperation::Jnp),
            0x71 => Some(JumpOperation::Jno),
            0x79 => Some(JumpOperation::Jns),
            0xe2 => Some(JumpOperation::Loop),
            0xe1 => Some(JumpOperation::Loopz),
            0xe0 => Some(JumpOperation::Loopnz),
            0xe3 => Some(JumpOperation::Jcxz),
            _ => None,
        }
    }
}

impl Display for JumpOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JumpOperation::Je => "je",
                JumpOperation::Jl => "jl",
                JumpOperation::Jle => "jle",
                JumpOperation::Jb => "jb",
                JumpOperation::Jbe => "jbe",
                JumpOperation::Jp => "jp",
                JumpOperation::Jo => "jo",
                JumpOperation::Js => "js",
                JumpOperation::Jne => "jne",
                JumpOperation::Jnl => "jnl",
                JumpOperation::Jnle => "jnle",
                JumpOperation::Jnb => "jnb",
                JumpOperation::Jnbe => "jnbe",
                JumpOperation::Jnp => "jnp",
                JumpOperation::Jno => "jno",
                JumpOperation::Jns => "jns",
                JumpOperation::Loop => "loop",
                JumpOperation::Loopz => "loopz",
                JumpOperation::Loopnz => "loopnz",
                JumpOperation::Jcxz => "jcxz",
            }
        )
    }
}

enum Instruction {
    Binary8 {
        operation: BinaryOperation,
        destination: Location,
        source: Operand8,
    },
    Binary16 {
        operation: BinaryOperation,
        destination: Location,
        source: Operand16,
    },
    Jump {
        operation: JumpOperation,
        displacement: i8,
    },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Binary8 {
                operation,
                destination,
                source,
            } => {
                let size = if let Location::Memory(_) = destination {
                    "byte "
                } else {
                    ""
                };
                write!(f, "{} {size}{}, {}", operation, destination, source)
            }
            Instruction::Binary16 {
                operation,
                destination,
                source,
            } => write!(f, "{} {}, {}", operation, destination, source),
            Instruction::Jump {
                operation,
                displacement,
            } => write!(f, "{} {}", operation, displacement),
        }
    }
}

fn decode_instruction(instructions: &[u8]) -> (Instruction, &[u8]) {
    let instruction = instructions[0];
    let (decoded_instruction, instruction_size) = if instruction >> 2 == 0b100010 {
        // register/memory to/from register
        decode_register_memory_instruction(instructions, BinaryOperation::Mov)
    } else if instruction >> 2 == 0 || instruction >> 2 == 0b001010 || instruction >> 2 == 0b001110
    {
        decode_register_memory_instruction(
            instructions,
            BinaryOperation::from_three_bit(instruction >> 3 & 7),
        )
    } else if instruction >> 4 == 0b1011 {
        // immediate to register
        let w = instruction >> 3 & 1 == 1;
        let reg = instruction & 7;

        if w {
            (
                Instruction::Binary16 {
                    operation: BinaryOperation::Mov,
                    destination: Location::Register(Register::from_reg_field(reg, true)),
                    source: Operand16::Immediate(
                        (instructions[1] as u16) | (instructions[2] as u16) << 8,
                    ),
                },
                3,
            )
        } else {
            (
                Instruction::Binary8 {
                    operation: BinaryOperation::Mov,
                    destination: Location::Register(Register::from_reg_field(reg, false)),
                    source: Operand8::Immediate(instructions[1]),
                },
                2,
            )
        }
    } else if instruction >> 1 == 0b1100011 {
        assert_eq!(instructions[1] >> 3 & 7, 0);
        decode_immediate_to_register_memory_instruction(instructions, BinaryOperation::Mov, false)
    } else if instruction >> 2 == 0b100000 {
        let s = instruction >> 1 & 1 == 1;
        decode_immediate_to_register_memory_instruction(
            instructions,
            BinaryOperation::from_three_bit(instructions[1] >> 3 & 7),
            s,
        )
    } else if instruction >> 2 == 0b101000 {
        let w = instruction & 1 == 1;
        let d = instruction >> 1 & 1 == 1;

        let address = if w {
            instructions[1] as u16 | (instructions[2] as u16) << 8
        } else {
            instructions[1] as u16
        };

        let decoded_instruction = {
            let register = Location::Register(Register::AX);
            let memory_location = Location::Memory(MemoryLocation::Direct(address));
            let (destination, source) = if d {
                (memory_location, register)
            } else {
                (register, memory_location)
            };
            Instruction::Binary16 {
                operation: BinaryOperation::Mov,
                destination,
                source: Operand16::Location(source),
            }
        };
        (decoded_instruction, if w { 3 } else { 2 })
    } else if instruction >> 1 == 0b10 || instruction >> 1 == 0b10110 || instruction >> 1 == 0b11110
    {
        let code = instruction >> 1;
        let operation = match code {
            0b10 => BinaryOperation::Add,
            0b10110 => BinaryOperation::Sub,
            0b11110 => BinaryOperation::Cmp,
            _ => unreachable!(),
        };
        decode_immediate_to_accumulator_instruction(instructions, operation)
    } else {
        decode_jump_instruction(instructions, JumpOperation::from_byte(instruction).unwrap())
    };

    (decoded_instruction, &instructions[instruction_size..])
}

fn make_displacement(bytes: &[u8]) -> i16 {
    match bytes {
        [first] => *first as i8 as i16,
        [first, second] => (*first as u16 | (*second as u16) << 8) as i16,
        _ => panic!(),
    }
}

// returns number of bytes in instruction
fn decode_register_memory_instruction(
    instructions: &[u8],
    operation: BinaryOperation,
) -> (Instruction, usize) {
    let d: bool = instructions[0] >> 1 & 1 > 0;
    let w: bool = instructions[0] & 1 == 1;
    let reg: u8 = instructions[1] >> 3 & 0b111;

    let instruction = if w {
        let register = Location::Register(Register::from_reg_field(reg, true));
        let register_memory = decode_register_memory(instructions, true);

        let (destination, source) = if d {
            (register, register_memory)
        } else {
            (register_memory, register)
        };
        Instruction::Binary16 {
            operation,
            destination,
            source: Operand16::Location(source),
        }
    } else {
        let register = Location::Register(Register::from_reg_field(reg, false));
        let register_memory = decode_register_memory(instructions, false);

        let (destination, source) = if d {
            (register, register_memory)
        } else {
            (register_memory, register)
        };
        Instruction::Binary8 {
            operation,
            destination,
            source: Operand8::Location(source),
        }
    };

    (
        instruction,
        2 + instruction_displacement_bytes(instructions),
    )
}

fn decode_immediate_to_register_memory_instruction(
    instructions: &[u8],
    operation: BinaryOperation,
    sign_extend: bool,
) -> (Instruction, usize) {
    let w = instructions[0] & 1 == 1;
    let data_start_index = 2 + instruction_displacement_bytes(instructions);

    let (instruction, data_size) = if w {
        let (immediate, data_size) = if sign_extend {
            (instructions[data_start_index] as i8 as u16, 1)
        } else {
            (
                (instructions[data_start_index] as u16)
                    | (instructions[data_start_index + 1] as u16) << 8,
                2,
            )
        };
        (
            Instruction::Binary16 {
                operation,
                destination: decode_register_memory(instructions, true),
                source: Operand16::Immediate(immediate),
            },
            data_size,
        )
    } else {
        (
            Instruction::Binary8 {
                operation,
                destination: decode_register_memory(instructions, false),
                source: Operand8::Immediate(instructions[data_start_index]),
            },
            1,
        )
    };

    (instruction, data_start_index + data_size)
}

fn decode_immediate_to_accumulator_instruction(
    instructions: &[u8],
    operation: BinaryOperation,
) -> (Instruction, usize) {
    let w = instructions[0] & 1 == 1;

    if w {
        (
            Instruction::Binary16 {
                operation,
                destination: Location::Register(Register::AX),
                source: Operand16::Immediate(
                    instructions[1] as u16 | (instructions[2] as u16) << 8,
                ),
            },
            3,
        )
    } else {
        (
            Instruction::Binary8 {
                operation,
                destination: Location::Register(Register::AL),
                source: Operand8::Immediate(instructions[1]),
            },
            2,
        )
    }
}

fn decode_jump_instruction(instructions: &[u8], operation: JumpOperation) -> (Instruction, usize) {
    (
        Instruction::Jump {
            operation,
            displacement: instructions[1] as i8,
        },
        2,
    )
}

fn decode_register_memory(instructions: &[u8], wide: bool) -> Location {
    let rm: u8 = instructions[1] & 0b111;
    let r#mod: u8 = instructions[1] >> 6 & 0b11;

    if r#mod == 0b11 {
        Location::Register(Register::from_reg_field(rm, wide))
    } else {
        Location::Memory(decode_memory_target(instructions))
    }
}

fn decode_memory_target(instructions: &[u8]) -> MemoryLocation {
    let rm: u8 = instructions[1] & 0b111;
    let r#mod: u8 = instructions[1] >> 6 & 0b11;

    if r#mod == 0b00 && rm == 0b110 {
        MemoryLocation::Direct(instructions[2] as u16 | (instructions[3] as u16) << 8)
    } else {
        let displacement = match r#mod {
            0b00 => 0,
            0b01 => make_displacement(&instructions[2..3]),
            0b10 => make_displacement(&instructions[2..4]),
            _ => unreachable!(),
        };
        MemoryLocation::Displaced {
            base: DisplacementBase::from_rm_field(rm),
            displacement,
        }
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
