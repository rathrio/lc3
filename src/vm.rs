use crate::op::{Op, Trap};
use std::io::Read;

const FL_POS: u16 = 0b001;
const FL_ZRO: u16 = 0b010;
const FL_NEG: u16 = 0b100;

/// Keyboard status
const MR_KBSR: u16 = 0xFE00;
/// Keyboard data
const MR_KBDR: u16 = 0xFE02;

/// In C, overflow is automatically handled. In Rust we'll have to be very explicit, so better
/// return u32 to avoid runtime errors.
fn sign_extend(mut x: u16, bit_count: u8) -> u32 {
    if (x >> (bit_count - 1)) & 1 != 0 {
        x |= 0xFFFF << bit_count
    }

    x as u32
}

fn read_char() -> u8 {
    std::io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .unwrap()
}

#[derive(Debug)]
pub struct VM {
    memory: [u16; u16::MAX as usize],
    registers: [u16; 8],
    running: bool,

    /// program counter
    pc: u16,

    /// condition flag
    cflag: u16,
}

impl VM {
    pub fn new(binary: Vec<u8>) -> Self {
        let mut memory = [0; u16::MAX as usize];

        let program: Vec<u16> = binary
            .chunks(2)
            // TIL macOS uses big endian...
            .map(|be_pair| (be_pair[0] as u16) << 8 | (be_pair[1] as u16))
            .collect();

        let origin = program.first().unwrap();
        let pc = *origin;

        program
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(index, instruction)| {
                let address = index + pc as usize;
                memory[address] = *instruction;
            });

        Self {
            memory,
            registers: [0; 8],
            running: true,
            pc,
            cflag: FL_ZRO,
        }
    }

    /// Load Effective Address
    ///
    /// ```encoding
    /// | 1110        | DR      | PCoffset9         |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// DR = PC + SEXT(PCoffset9);
    /// setcc();
    /// ```
    fn lea(&mut self, instr: u16) {
        let dr = (instr >> 9) & 0b111;
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        self.write_register(dr, (self.pc as u32 + pc_offset) as u16);
    }

    /// Addition
    ///
    /// ```encoding
    /// | 0001        | DR      | SR1   | 0 |     | SR2   |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 | 4 3 | 2 1 0 |
    ///
    /// | 0001        | DR      | SR1   | 1 | imm5      |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 | 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// if (bit[5] == 0)
    ///     DR = SR1 + SR2;
    /// else
    ///     DR = SR1 + SEXT(imm5);
    /// setcc();
    /// ```
    fn add(&mut self, instr: u16) {
        let dr = (instr >> 9) & 0x7;
        let r1 = (instr >> 6) & 0x7;
        let imm_flag = (instr >> 5) & 1;

        if imm_flag == 1 {
            let imm5 = sign_extend(instr & 0x1F, 5);
            let result = (self.read_register(r1) as u32 + imm5) as u16;
            self.write_register(dr, result);
        } else {
            let r2 = instr & 0b111;
            self.write_register(
                dr,
                (self.read_register(r1) as u32 + self.read_register(r2) as u32) as u16,
            );
        }
    }

    /// System Call
    ///
    /// ```encoding
    /// | 1111        |           | trapvect8       |
    /// | 15 14 13 12 | 11 10 9 8 | 7 6 5 4 3 2 1 0 |
    /// ```
    fn trap(&mut self, instr: u16) {
        let trap = instr & 0xff;
        // let t: Trap = trap.try_into().unwrap();
        // dbg!(t);
        match trap.try_into() {
            Ok(Trap::PUTS) => self.puts(),
            Ok(Trap::HALT) => self.halt(),
            Ok(Trap::GETC) => self.getc(),
            Ok(Trap::OUT) => self.out(),
            Ok(Trap::IN) => self.t_in(),
            Ok(Trap::PUTSP) => self.putsp(),
            Err(_) => (),
        }
    }

    fn puts(&self) {
        let mut address = self.registers[0] as usize;
        let mut string = String::from("");

        while self.memory[address] != 0 {
            string.push(self.memory[address] as u8 as char);
            address += 1;
        }

        print!("{}", string);
    }

    fn halt(&mut self) {
        println!("\n{}", "HALT");
        self.running = false;
    }

    fn getc(&mut self) {
        self.write_register(0, read_char() as u16);
    }

    fn out(&self) {
        print!("{}", self.read_register(0) as u8 as char);
    }

    fn t_in(&mut self) {
        print!("Enter a character: ");
        let c = read_char();
        println!("{}", c as char);
        self.write_register(0, c as u16);
    }

    fn putsp(&self) {
        todo!();
    }

    /// Load
    ///
    /// ```encoding
    /// | 0010        | DR      | PCoffset9         |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// DR = mem[PC + SEXT(PCoffset9)];
    /// setcc();
    /// ```
    fn ld(&mut self, instr: u16) {
        let dr = (instr >> 9) & 0x7;
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let result = self.read_memory((self.pc as u32 + pc_offset) as u16);
        self.write_register(dr, result);
    }

    /// Jump to Subroutine
    ///
    /// JRS
    ///
    /// ```encoding
    /// | 0100        | 1  | PCoffset11             |
    /// | 15 14 13 12 | 11 | 10 9 8 7 6 5 4 3 2 1 0 |
    /// ```
    ///
    /// JSRR
    ///
    /// ```encoding
    /// | 0100        | 0  |      | BaseR |             |
    /// | 15 14 13 12 | 11 | 10 9 | 8 7 6 | 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// R7 = PC;
    /// if (bit[11] == 0)
    ///     PC = BaseR;
    /// else
    ///     PC = PC + SEXT(PCoffset11);
    /// ```
    fn jsr(&mut self, instr: u16) {
        self.write_register(7, self.pc);
        let jsr_flag = (instr >> 11) & 1;
        if jsr_flag == 1 {
            let target = self.pc as u32 + sign_extend(instr & 0x7FF, 11);
            self.pc = target as u16;
        } else {
            // JSRR
            self.pc = self.read_register((instr >> 6) & 0x7);
        }
    }

    /// Store Base + offset
    ///
    /// ```encoding
    /// | 0111        | SR      | BaseR | offset6     |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// mem[BaseR + SEXT(offset6)] = SR;
    /// ```
    fn str(&mut self, instr: u16) {
        let sr = self.read_register((instr >> 9) & 0x7);
        let base_r = self.read_register((instr >> 6) & 0x7) as u32;
        let offset = sign_extend(instr & 0x3F, 6);
        self.write_memory((base_r + offset) as u16, sr);
    }

    /// Conditional Branch
    ///
    /// ```encoding
    /// | 0000        | n  z  p | PCoffset9         |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// if ((n AND N) OR (z AND Z) OR (p AND P))
    ///     PC = PC + SEXT(PCoffset9);
    /// ```
    fn br(&mut self, instr: u16) {
        let offset = sign_extend(instr & 0x1FF, 9);
        let cond_flag = (instr >> 9) & 0x7;
        if (cond_flag & self.cflag) != 0 {
            let target = self.pc as u32 + offset;
            self.pc = target as u16;
        }
    }

    /// Bit-Wise Logical AND
    ///
    /// ```encoding
    /// | 0101        | DR      | SR1   | 0 |     | SR2   |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 | 4 3 | 2 1 0 |
    ///
    /// | 0101        | DR      | SR1   | 1 | imm5      |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 | 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// if (bit[5] == 0)
    ///     DR = SR1 AND SR2;
    /// else
    ///     DR = SR1 AND SEXT(imm5);
    /// setcc();
    /// ```
    fn and(&mut self, instr: u16) {
        let dr = (instr >> 9) & 0x7;
        let sr1 = (instr >> 6) & 0x7;
        let flag = (instr >> 5) & 1;

        if flag == 1 {
            let imm5 = instr & 0x1F;
            let result = self.read_register(sr1) & sign_extend(imm5, 5) as u16;
            self.write_register(dr, result);
        } else {
            let sr2 = instr & 0x7;
            self.write_register(dr, self.read_register(sr1) & self.read_register(sr2));
        }
    }

    /// Store
    ///
    /// ```encoding
    /// | 0011        | SR      | PCoffset9         |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// mem[PC + SEXT(PCoffset9)] = SR;
    /// ```
    fn st(&mut self, instr: u16) {
        let sr = (instr >> 9) & 0x7;
        let offset = sign_extend(instr & 0x1FF, 9);
        self.write_memory((self.pc as u32 + offset) as u16, self.read_register(sr));
    }

    /// Load Base + offset
    ///
    /// ```encoding
    /// | 0110        | DR      | BaseR | offset6     |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// DR = mem[BaseR + SEXT(offset6)];
    /// setcc();
    /// ```
    fn ldr(&mut self, instr: u16) {
        let dr = (instr >> 9) & 0x7;
        let base_r = (instr >> 6) & 0x7;
        let offset = instr & 0x3F;
        let loaded =
            self.read_memory((self.read_register(base_r) as u32 + sign_extend(offset, 6)) as u16);

        self.write_register(dr, loaded);
    }

    /// Jump
    ///
    /// ```encoding
    /// | 1100        |         | BaseR |             |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0 |
    /// ```
    ///
    /// Return from Subroutine
    ///
    /// ```encoding
    /// | 1100        |         | 1 1 1 |             |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// PC = BaseR;
    /// ```
    fn jmp(&mut self, instr: u16) {
        let base_r = (instr >> 6) & 0x7;
        self.pc = self.read_register(base_r);
    }

    /// Bit-Wise  Complement
    ///
    /// ```encoding
    /// | 1001        | DR      | SR    |             |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// DR = NOT(SR);
    /// setcc();
    /// ```
    fn not(&mut self, instr: u16) {
        let dr = (instr >> 9) & 0x7;
        let sr = (instr >> 6) & 0x7;
        self.write_register(dr, !self.read_register(sr));
    }

    /// Load Indirect
    ///
    /// ```encoding
    /// | 1010        | DR      | PCoffset9         |
    /// | 15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0 |
    /// ```
    ///
    /// ```operation
    /// DR = mem[mem[PC + SEXT(PCoffset9)]];
    /// setcc();
    /// ```
    fn ldi(&mut self, instr: u16) {
        let dr = (instr >> 9) & 0x7;
        let offset = sign_extend(instr & 0x1FF, 9);
        let address_of_address = (self.pc as u32 + offset) as u16;
        let address = self.read_memory(address_of_address);
        let result = self.read_memory(address);
        self.write_register(dr, result);
    }

    pub fn run(&mut self) {
        loop {
            if !self.running {
                break;
            }

            let instr = self.read_memory(self.pc);
            self.pc += 1;

            let op = instr >> 12;
            match op.try_into() {
                Ok(Op::LEA) => self.lea(instr),
                Ok(Op::ADD) => self.add(instr),
                Ok(Op::TRAP) => self.trap(instr),
                Ok(Op::LD) => self.ld(instr),
                Ok(Op::JSR) => self.jsr(instr),
                Ok(Op::BR) => self.br(instr),
                Ok(Op::ST) => self.st(instr),
                Ok(Op::STR) => self.str(instr),
                Ok(Op::AND) => self.and(instr),
                Ok(Op::LDR) => self.ldr(instr),
                Ok(Op::JMP) => self.jmp(instr),
                Ok(Op::NOT) => self.not(instr),
                Ok(Op::LDI) => self.ldi(instr),
                Ok(o) => panic!("Did not implement op {:#?} ({:#06b})", o, op),
                Err(_) => (),
            }
        }
    }

    fn write_register(&mut self, register: u16, value: u16) {
        self.registers[register as usize] = value;

        if value == 0 {
            self.cflag = FL_ZRO;
        } else if value >> 15 == 1 {
            self.cflag = FL_NEG;
        } else {
            self.cflag = FL_POS;
        }
    }

    fn read_register(&self, register: u16) -> u16 {
        self.registers[register as usize]
    }

    fn read_memory(&mut self, address: u16) -> u16 {
        if address == MR_KBSR {
            let c = read_char();
            if c != 0 {
                self.write_memory(MR_KBSR, 1 << 15);
                self.write_memory(MR_KBDR, c as u16);
            } else {
                self.write_memory(MR_KBSR, 0);
            }
        }
        self.memory[address as usize]
    }

    fn write_memory(&mut self, address: u16, value: u16) {
        self.memory[address as usize] = value;
    }
}
