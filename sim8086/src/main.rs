use std::fmt;
use std::fmt::Display;
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    match command.as_deref() {
        Some("execute") => {
            execute();
            ExitCode::SUCCESS
        }
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

fn execute() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let instructions = input.as_slice();

    let mut processor = Box::new(Processor::new());

    println!("bits 16");

    while (processor.ip as usize) < instructions.len() {
        let ip_before = processor.ip;
        let sf_before = processor.sf;
        let zf_before = processor.zf;

        let (instruction, size) = decode_instruction(&instructions[(processor.ip as usize)..]);
        processor.ip += size;

        let before = if let Instruction::Binary {
            destination: Location::Register(register),
            ..
        } = instruction
        {
            Some(processor.read_register(register))
        } else {
            None
        };
        processor.execute(instruction);

        print!("{};", instruction);

        if let Instruction::Binary {
            destination: Location::Register(register),
            ..
        } = instruction
        {
            let before = before.unwrap();
            let after = processor.read_register(register);
            print!(" {register}:{before:#x}->{after:#x}");
        };

        print!(" ip:{ip_before:#x}->{:#x}", processor.ip);

        if let Instruction::Binary {
            operation: BinaryOperation::Add | BinaryOperation::Sub | BinaryOperation::Cmp,
            ..
        } = instruction
        {
            print!(" flags:");
            if sf_before {
                print!("S");
            }
            if zf_before {
                print!("Z");
            }
            print!("->");
            if processor.sf {
                print!("S");
            }
            if processor.zf {
                print!("Z");
            }
        }

        println!();
    }

    println!("\nFinal registers:");
    for register in [
        Register::AX,
        Register::BX,
        Register::CX,
        Register::DX,
        Register::SP,
        Register::BP,
        Register::SI,
        Register::DI,
    ] {
        println!(
            "    {register}: {:#06x} ({0:})",
            processor.read_register(register)
        );
    }
    println!("    ip: {:#06x}", processor.ip);
    print!(" flags: ");
    if processor.sf {
        print!("S");
    }
    if processor.zf {
        print!("Z");
    }
    println!();
}

// reads instructions on stdin and output disassembled instructions to stdout
fn decode() {
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap();
    let mut instructions = input.as_slice();

    println!("bits 16");

    while !instructions.is_empty() {
        let (instruction, size) = decode_instruction(instructions);
        println!("{}", instruction);
        instructions = &instructions[(size as usize)..]
    }
}

struct Processor {
    ax: u16,
    bx: u16,
    cx: u16,
    dx: u16,
    sp: u16,
    bp: u16,
    si: u16,
    di: u16,
    ip: u16,
    sf: bool, // sign flag
    zf: bool, // zero flag
    memory: [u8; 0x10000],
}

impl Processor {
    fn new() -> Self {
        Processor {
            ax: 0,
            bx: 0,
            cx: 0,
            dx: 0,
            sp: 0,
            bp: 0,
            si: 0,
            di: 0,
            ip: 0,
            sf: false,
            zf: false,
            memory: [0; 0x10000],
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Binary {
                operation,
                destination,
                source,
                wide,
            } => {
                let source_value = match source {
                    Operand::Immediate(value) => value,
                    Operand::Location(location) => self.read_location(location),
                };

                let destination_value = self.read_location(destination);

                let (result_value, update_flags) = match operation {
                    BinaryOperation::Mov => (source_value, false),
                    BinaryOperation::Add => (destination_value.wrapping_add(source_value), true),
                    BinaryOperation::Sub | BinaryOperation::Cmp => {
                        (destination_value.wrapping_sub(source_value), true)
                    }
                };

                if update_flags {
                    self.sf = result_value >> 15 & 1 == 1;
                    self.zf = result_value == 0;
                }

                if operation != BinaryOperation::Cmp {
                    self.write_location(destination, result_value, wide)
                }
            }
            _ => unimplemented!(),
        }
    }

    fn read_location(&self, location: Location) -> u16 {
        match location {
            Location::Register(register) => self.read_register(register),
            Location::Memory(memory_location) => self.read_memory_location(memory_location),
        }
    }

    fn read_memory_location(&self, memory_location: MemoryLocation) -> u16 {
        self.read_memory_address(self.memory_location_to_address(memory_location))
    }

    fn memory_location_to_address(&self, memory_location: MemoryLocation) -> u16 {
        match memory_location {
            MemoryLocation::Displaced { base, displacement } => {
                (self.read_displacement_base(base) as i32 + displacement as i32) as u16
            }
            MemoryLocation::Direct(address) => address,
        }
    }

    fn read_memory_address(&self, address: u16) -> u16 {
        self.memory[address as usize] as u16
            | (self.memory.get(address as usize + 1).copied().unwrap_or(0) as u16) << 8
    }

    fn read_register(&self, register: Register) -> u16 {
        match register {
            Register::AL => self.ax & 0xff,
            Register::CL => self.cx & 0xff,
            Register::DL => self.dx & 0xff,
            Register::BL => self.bx & 0xff,
            Register::AH => self.ax >> 8,
            Register::CH => self.cx >> 8,
            Register::DH => self.dx >> 8,
            Register::BH => self.bx >> 8,
            Register::AX => self.ax,
            Register::CX => self.cx,
            Register::DX => self.dx,
            Register::BX => self.bx,
            Register::SP => self.sp,
            Register::BP => self.bp,
            Register::SI => self.si,
            Register::DI => self.di,
        }
    }

    fn read_displacement_base(&self, displacement_base: DisplacementBase) -> u16 {
        match displacement_base {
            DisplacementBase::Bxsi => self.bx.wrapping_add(self.si),
            DisplacementBase::Bxdi => self.bx.wrapping_add(self.di),
            DisplacementBase::Bpsi => self.bp.wrapping_add(self.si),
            DisplacementBase::Bpdi => self.bp.wrapping_add(self.di),
            DisplacementBase::Si => self.si,
            DisplacementBase::Di => self.di,
            DisplacementBase::Bp => self.bp,
            DisplacementBase::Bx => self.bx,
        }
    }

    fn write_location(&mut self, location: Location, value: u16, wide: bool) {
        match location {
            Location::Register(register) => self.write_register(register, value),
            Location::Memory(memory_location) => {
                self.write_memory_location(memory_location, value, wide)
            }
        }
    }

    fn write_memory_location(&mut self, memory_location: MemoryLocation, value: u16, wide: bool) {
        self.write_memory_address(
            self.memory_location_to_address(memory_location),
            value,
            wide,
        )
    }

    fn write_memory_address(&mut self, address: u16, value: u16, wide: bool) {
        let low = value as u8;
        if wide {
            let high = (value >> 8) as u8;
            self.memory[address as usize] = low;
            self.memory.get_mut(address as usize + 1).map(|m| *m = high);
        } else {
            self.memory[address as usize] = low;
        }
    }

    fn write_register(&mut self, register: Register, value: u16) {
        match register {
            Register::AL => {
                assert!(value < 0x100);
                self.ax = self.ax & 0xff00 | value;
            }
            Register::CL => {
                assert!(value < 0x100);
                self.cx = self.cx & 0xff00 | value;
            }
            Register::DL => {
                assert!(value < 0x100);
                self.dx = self.dx & 0xff00 | value;
            }
            Register::BL => {
                assert!(value < 0x100);
                self.bx = self.bx & 0xff00 | value;
            }
            Register::AH => {
                assert!(value < 0x100);
                self.ax = self.ax & 0xff | value << 8;
            }
            Register::CH => {
                assert!(value < 0x100);
                self.cx = self.cx & 0xff | value << 8;
            }
            Register::DH => {
                assert!(value < 0x100);
                self.dx = self.dx & 0xff | value << 8;
            }
            Register::BH => {
                assert!(value < 0x100);
                self.bx = self.bx & 0xff | value << 8;
            }
            Register::AX => self.ax = value,
            Register::CX => self.cx = value,
            Register::DX => self.dx = value,
            Register::BX => self.bx = value,
            Register::SP => self.sp = value,
            Register::BP => self.bp = value,
            Register::SI => self.si = value,
            Register::DI => self.di = value,
        }
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
enum Operand {
    Immediate(u16),
    Location(Location),
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Immediate(s) => write!(f, "{}", s),
            Operand::Location(s) => write!(f, "{}", s),
        }
    }
}

// instructions
#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy)]
enum Instruction {
    Binary {
        operation: BinaryOperation,
        destination: Location,
        source: Operand,
        wide: bool,
    },
    Jump {
        operation: JumpOperation,
        displacement: i8,
    },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Binary {
                operation,
                destination,
                source,
                wide,
            } => {
                let size = if let Location::Memory(_) = destination {
                    if *wide { "word " } else { "byte " }
                } else {
                    ""
                };
                write!(f, "{} {size}{}, {}", operation, destination, source)
            }
            Instruction::Jump {
                operation,
                displacement,
            } => write!(f, "{} {}", operation, displacement),
        }
    }
}

fn decode_instruction(instructions: &[u8]) -> (Instruction, u16) {
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

        let (immediate, instruction_size) = if w {
            ((instructions[1] as u16) | (instructions[2] as u16) << 8, 3)
        } else {
            (instructions[1] as u16, 2)
        };

        (
            Instruction::Binary {
                operation: BinaryOperation::Mov,
                destination: Location::Register(Register::from_reg_field(reg, w)),
                source: Operand::Immediate(immediate),
                wide: w,
            },
            instruction_size,
        )
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
            Instruction::Binary {
                operation: BinaryOperation::Mov,
                destination,
                source: Operand::Location(source),
                wide: true,
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

    (decoded_instruction, instruction_size)
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
) -> (Instruction, u16) {
    let d: bool = instructions[0] >> 1 & 1 > 0;
    let w: bool = instructions[0] & 1 == 1;
    let reg: u8 = instructions[1] >> 3 & 0b111;

    let register = Location::Register(Register::from_reg_field(reg, w));
    let register_memory = decode_register_memory(instructions, w);

    let (destination, source) = if d {
        (register, register_memory)
    } else {
        (register_memory, register)
    };

    (
        Instruction::Binary {
            operation,
            destination,
            source: Operand::Location(source),
            wide: w,
        },
        2 + instruction_displacement_bytes(instructions),
    )
}

fn decode_immediate_to_register_memory_instruction(
    instructions: &[u8],
    operation: BinaryOperation,
    sign_extend: bool,
) -> (Instruction, u16) {
    let w = instructions[0] & 1 == 1;
    let data_start_index = 2 + instruction_displacement_bytes(instructions);

    let (immediate, data_size) = if w {
        if sign_extend {
            (instructions[data_start_index as usize] as i8 as u16, 1)
        } else {
            (
                (instructions[data_start_index as usize] as u16)
                    | (instructions[(data_start_index + 1) as usize] as u16) << 8,
                2,
            )
        }
    } else {
        (instructions[data_start_index as usize] as u16, 1)
    };

    (
        Instruction::Binary {
            operation,
            destination: decode_register_memory(instructions, w),
            source: Operand::Immediate(immediate),
            wide: w,
        },
        (data_start_index + data_size) as u16,
    )
}

fn decode_immediate_to_accumulator_instruction(
    instructions: &[u8],
    operation: BinaryOperation,
) -> (Instruction, u16) {
    let w = instructions[0] & 1 == 1;

    let (register, immediate, instruction_size) = if w {
        (
            Register::AX,
            instructions[1] as u16 | (instructions[2] as u16) << 8,
            3,
        )
    } else {
        (Register::AL, instructions[1] as u16, 2)
    };

    (
        Instruction::Binary {
            operation,
            destination: Location::Register(register),
            source: Operand::Immediate(immediate),
            wide: w,
        },
        instruction_size,
    )
}

fn decode_jump_instruction(instructions: &[u8], operation: JumpOperation) -> (Instruction, u16) {
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

fn instruction_displacement_bytes(instructions: &[u8]) -> u16 {
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
