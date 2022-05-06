#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// https://www.masswerk.at/6502/6502_instruction_set.html
// http://www.6502.org/tutorials/6502opcodes.html

use crate::bus::Bus;
use crate::cpu::AddressingMode::{
    Absolute, AbsoluteX, AbsoluteY, Accumulator, Immediate, IndirectX, IndirectY, ZeroPage,
    ZeroPageX, ZeroPageY,
};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Add;

const STACK_BYTE_HIGH: u16 = 0x0100;

enum Flag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    IntDisable = 0b0000_0100,
    Decimal = 0b0000_1000,
    Break = 0b0001_0000,
    Unused = 0b0010_0000,
    Overflow = 0b0100_0000,
    Negative = 0b1000_0000,
}

pub struct Cpu {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    p: u8,
    bus: Bus,
}

#[derive(Debug)]
enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    Immediate,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

#[derive(Debug)]
enum Opcode {
    Adc(AddressingMode),
    And(AddressingMode),
    Asl(AddressingMode),
    Bcc,
    Bcs,
    Beq,
    Bit(AddressingMode),
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp(AddressingMode),
    Cpx(AddressingMode),
    Cpy(AddressingMode),
    Dec(AddressingMode),
    Dex,
    Dey,
    Eor(AddressingMode),
    Inc(AddressingMode),
    Inx,
    Iny,
    Jmp(AddressingMode),
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora(AddressingMode),
    Pha,
    Php,
    Pla,
    Plp,
    Rol(AddressingMode),
    Ror(AddressingMode),
    Rti,
    Rts,
    Sbc(AddressingMode),
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Cpu {
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            bus,
        }
    }

    fn set_zero_negative_flags(&mut self, value: u8) {
        if value == 0 {
            self.p |= Flag::Zero as u8
        };
        if (value & 0x80) > 0 {
            self.p |= Flag::Negative as u8
        }
    }

    fn clear_flag(&mut self, flag: Flag) {
        self.p &= !(flag as u8);
    }

    fn set_flag(&mut self, flag: Flag) {
        self.p |= flag as u8;
    }

    fn change_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    fn is_flag_set(&self, flag: Flag) -> bool {
        self.p & flag as u8 != 0
    }

    fn is_flag_clear(&self, flag: Flag) -> bool {
        self.p & flag as u8 == 0
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            self.pc = self.get_operand(AddressingMode::Relative);
        }
    }

    fn compare(&mut self, mode: AddressingMode, reg: u8) {
        let op = (self.get_operand(mode) & 0xff) as u8;
        let result = reg.wrapping_sub(op);
        self.change_flag(Flag::Carry, reg >= op);
        self.set_zero_negative_flags(result);
    }

    fn stack_push_u8(&mut self, value: u8) {
        self.bus.write_u8(STACK_BYTE_HIGH | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_push_u16(&mut self, value: u16) {
        self.stack_push_u8((value & 0xff) as u8);
        self.stack_push_u8(((value & 0xff00) >> 8) as u8);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read_u8(STACK_BYTE_HIGH | self.sp as u16)
    }

    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::Nop => {}

            // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
            Opcode::Adc(mode) => {
                let carry: u8 = if self.is_flag_set(Flag::Carry) { 1 } else { 0 };
                let operand = self.get_operand(mode) as u8;
                let result = self.a.wrapping_add(operand.wrapping_add(carry));
                self.change_flag(
                    Flag::Overflow,
                    (operand ^ result) & (self.a ^ result) & 0x80 != 0,
                );
                self.a = result;
                self.set_zero_negative_flags(self.a);
            }

            Opcode::And(mode) => {
                self.a = self.a & (self.get_operand(mode) as u8);
                self.set_zero_negative_flags(self.a)
            }
            Opcode::Asl(mode) => {
                let op = self.get_operand(mode);
                self.a = op.rotate_left(1) as u8 & 0xfe;
                if op & 0x80 != 0 {
                    self.set_flag(Flag::Carry);
                } else {
                    self.clear_flag(Flag::Carry);
                }
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Bcc => {
                self.branch(self.is_flag_clear(Flag::Carry));
            }
            Opcode::Bcs => {
                self.branch(self.is_flag_set(Flag::Carry));
            }
            Opcode::Bvc => {
                self.branch(self.is_flag_clear(Flag::Overflow));
            }
            Opcode::Bvs => {
                self.branch(self.is_flag_set(Flag::Overflow));
            }
            Opcode::Bne => {
                self.branch(self.is_flag_clear(Flag::Zero));
            }
            Opcode::Beq => {
                self.branch(self.is_flag_set(Flag::Zero));
            }
            Opcode::Bpl => {
                self.branch(self.is_flag_clear(Flag::Negative));
            }
            Opcode::Bmi => {
                self.branch(self.is_flag_set(Flag::Negative));
            }
            Opcode::Clc => {
                self.clear_flag(Flag::Carry);
            }
            Opcode::Sec => {
                self.set_flag(Flag::Carry);
            }
            Opcode::Cli => {
                self.clear_flag(Flag::IntDisable);
            }
            Opcode::Sei => {
                self.set_flag(Flag::IntDisable);
            }
            Opcode::Clv => {
                self.clear_flag(Flag::Overflow);
            }
            Opcode::Cld => {
                self.clear_flag(Flag::Decimal);
            }
            Opcode::Sed => {
                self.set_flag(Flag::Decimal);
            }
            Opcode::Inc(mode) => {
                let addr = self.get_operand_address(mode);
                let value = self.bus.read_u8(addr).wrapping_add(1);
                self.bus.write_u8(addr, value);
            }
            Opcode::Inx => {
                self.x = self.x.wrapping_add(1);
                self.set_zero_negative_flags(self.x);
            }
            Opcode::Iny => {
                self.y = self.y.wrapping_add(1);
                self.set_zero_negative_flags(self.y);
            }
            Opcode::Bit(mode) => {
                let result = self.a & self.get_operand(mode) as u8;
                self.set_zero_negative_flags(result);
                self.change_flag(Flag::Overflow, result & 0x40 != 0);
                // TODO write tests
            }
            Opcode::Brk => {
                self.pc = self.pc.wrapping_add(1);
                self.set_flag(Flag::IntDisable);
                self.stack_push_u16(self.pc);
                self.stack_push_u8(self.p);
                self.pc = self.bus.read_u16(0xfffe);
                // TODO write tests
            }
            Opcode::Jsr => {
                let tgt_addr = self.get_operand_address(Absolute);
                let ret_addr = self.pc - 1;
                self.stack_push_u16(ret_addr);
                self.pc = tgt_addr;
                // TODO write tests
            }
            Opcode::Cmp(mode) => {
                self.compare(mode, self.a);
                // TODO write tests
            }
            Opcode::Cpx(mode) => {
                self.compare(mode, self.x);
                // TODO write tests
            }
            Opcode::Cpy(mode) => {
                self.compare(mode, self.y);
                // TODO write tests
            }
            Opcode::Dec(mode) => {
                let addr = self.get_operand_address(mode);
                let value = self.bus.read_u8(addr).wrapping_sub(1);
                self.bus.write_u8(addr, value);
                // TODO write tests
            }
            Opcode::Dex => {
                self.x = self.x.wrapping_sub(1);
                self.set_zero_negative_flags(self.x);
            }
            Opcode::Dey => {
                self.y = self.y.wrapping_sub(1);
                self.set_zero_negative_flags(self.y);
            }
            Opcode::Eor(mode) => {
                self.a = self.a ^ (self.get_operand(mode) as u8);
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Ora(mode) => {
                self.a = self.a | (self.get_operand(mode) as u8);
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Pha => {
                self.stack_push_u8(self.a);
            }
            Opcode::Php => {
                self.stack_push_u8(self.p);
            }
            Opcode::Pla => {
                self.a = self.stack_pop();
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Plp => {
                self.p = self.stack_pop();
            }
            Opcode::Rol(mode) => {
                let carry_in = if self.p & (Flag::Carry as u8) != 0 {
                    0x01
                } else {
                    0x00
                };
                let op = self.get_operand(mode);
                if op & 0x80 != 0 {
                    self.set_flag(Flag::Carry);
                } else {
                    self.clear_flag(Flag::Carry);
                }
                self.a = op.rotate_left(1) as u8 | carry_in;
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Ror(mode) => {
                let carry_in = if self.p & (Flag::Carry as u8) != 0 {
                    0x80
                } else {
                    0x00
                };
                let op = self.get_operand(mode);
                if op & 0x01 != 0 {
                    self.set_flag(Flag::Carry);
                } else {
                    self.clear_flag(Flag::Carry);
                }
                self.a = op.rotate_right(1) as u8 | carry_in;
                self.set_zero_negative_flags(self.a);
            }

            Opcode::Sbc(mode) => {
                let carry: u8 = if self.is_flag_clear(Flag::Carry) {
                    1
                } else {
                    0
                };
                let operand = self.get_operand(mode) as u8;
                let result = self.a.wrapping_sub(operand.wrapping_add(carry));
                self.change_flag(
                    Flag::Overflow,
                    (operand ^ result) & (self.a ^ result) & 0x80 != 0,
                );
                self.a = result;
                self.set_zero_negative_flags(self.a);
            }

            Opcode::Tax => {
                self.x = self.a;
                self.set_zero_negative_flags(self.x);
            }
            Opcode::Tay => {
                self.y = self.a;
                self.set_zero_negative_flags(self.y);
            }
            Opcode::Tsx => {
                self.x = self.sp;
                self.set_zero_negative_flags(self.x);
            }
            Opcode::Txa => {
                self.a = self.x;
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Txs => {
                self.sp = self.x;
                self.set_zero_negative_flags(self.sp);
            }
            Opcode::Tya => {
                self.a = self.y;
                self.set_zero_negative_flags(self.a);
            }

            Opcode::Jmp(mode) => {
                self.pc = self.get_operand(mode);
            }
            _ => {
                panic!("unimplemented opcode: {}", opcode);
            }
        }
    }

    /// Returns the address of the operand given the addressing mode. Some mods such as Immediate and Accumulator will return 0 as an invalid state
    fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            Immediate => self.pc,
            ZeroPage => 0x0000 + self.bus.read_u8(self.pc) as u16,
            ZeroPageX => 0x0000 + self.bus.read_u8(self.pc).wrapping_add(self.x) as u16,
            ZeroPageY => 0x0000 + self.bus.read_u8(self.pc).wrapping_add(self.y) as u16,
            Absolute => self.pc,
            AbsoluteX => self.pc.wrapping_add(self.x as u16),
            AbsoluteY => self.pc.wrapping_add(self.y as u16),

            AddressingMode::Indirect => self.bus.read_u16(self.pc),
            IndirectX => self.bus.read_u16(self.pc).wrapping_add(self.x as u16),
            IndirectY => self.bus.read_u16(self.pc).wrapping_add(self.y as u16),
            _ => 0,
        }
    }

    /// Given the current state of the pc and addressing mode, this function will return the appropriate operand
    /// reference: https://www.nesdev.org/wiki/CPU_addressing_modes
    fn get_operand(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Relative => {
                let offset = self.bus.read_u8(self.pc) as i8;
                if offset > 0 {
                    self.pc.wrapping_add(offset as u16)
                } else {
                    self.pc.wrapping_sub(offset as u16)
                }
            }
            Accumulator => self.a as u16,
            _ => self.bus.read_u16(self.get_operand_address(mode)),
        }
    }

    /// decode takes in an opcode and outputs a tuple with opcode/addressing mode and cycles
    /// reference: https://www.nesdev.org/obelisk-6502-guide/reference.html
    fn decode(&self, opcode: u8) -> (Opcode, u8) {
        match opcode {
            // ADC (Add Memory to Accumulator with Carry)
            0x69 => (Opcode::Adc(Immediate), 2),
            0x65 => (Opcode::Adc(ZeroPage), 3),
            0x75 => (Opcode::Adc(ZeroPageX), 4),
            0x6d => (Opcode::Adc(Absolute), 4),
            0x7d => (Opcode::Adc(AbsoluteX), 4),
            0x79 => (Opcode::Adc(AbsoluteY), 4),
            0x61 => (Opcode::Adc(IndirectX), 6),
            0x71 => (Opcode::Adc(IndirectY), 5),

            // AND (bitwise AND with accumulator)
            0x29 => (Opcode::And(Immediate), 2),
            0x25 => (Opcode::And(ZeroPage), 3),
            0x35 => (Opcode::And(ZeroPageX), 4),
            0x2d => (Opcode::And(Absolute), 4),
            0x3d => (Opcode::And(AbsoluteX), 4),
            0x39 => (Opcode::And(AbsoluteY), 4),
            0x21 => (Opcode::And(IndirectX), 6),
            0x31 => (Opcode::And(IndirectY), 5),

            // ASL (Arithmetic Shift Left)
            0x0a => (Opcode::Asl(Accumulator), 2),
            0x06 => (Opcode::Asl(ZeroPage), 5),
            0x16 => (Opcode::Asl(ZeroPageX), 6),
            0x0e => (Opcode::Asl(Absolute), 2),
            0x1e => (Opcode::Asl(AbsoluteX), 4),

            // Branch instructions
            0x90 => (Opcode::Bcc, 2),
            0xb0 => (Opcode::Bcs, 2),
            0xf0 => (Opcode::Beq, 2),
            0x30 => (Opcode::Bmi, 2),
            0xd0 => (Opcode::Bne, 2),
            0x10 => (Opcode::Bpl, 2),

            // BIT (test BITs in Memory With Accumulator)
            0x24 => (Opcode::Bit(ZeroPage), 3),
            0x2c => (Opcode::Bit(Absolute), 4),

            // BRK (Force break)
            0x00 => (Opcode::Brk, 7),

            // Flag operations
            0x18 => (Opcode::Clc, 2),
            0x38 => (Opcode::Sec, 2),
            0x58 => (Opcode::Cli, 2),
            0x78 => (Opcode::Sei, 2),
            0xb8 => (Opcode::Clv, 2),
            0xd8 => (Opcode::Cld, 2),
            0xf8 => (Opcode::Sed, 2),

            // CMP (Compare Memory with Accumulator)
            0xc9 => (Opcode::Cmp(Immediate), 2),
            0xc5 => (Opcode::Cmp(ZeroPage), 3),
            0xd5 => (Opcode::Cmp(ZeroPageX), 4),
            0xcd => (Opcode::Cmp(Absolute), 4),
            0xdd => (Opcode::Cmp(AbsoluteX), 4),
            0xd9 => (Opcode::Cmp(AbsoluteY), 4),
            0xc1 => (Opcode::Cmp(IndirectX), 6),
            0xd1 => (Opcode::Cmp(IndirectY), 5),

            // CPX (Compare Memory and Index X)
            0xe0 => (Opcode::Cpx(Immediate), 2),
            0xe4 => (Opcode::Cpx(ZeroPage), 3),
            0xec => (Opcode::Cpx(Absolute), 4),

            // CPY (Compare Memory and Index Y)
            0xc0 => (Opcode::Cpy(Immediate), 2),
            0xc4 => (Opcode::Cpy(ZeroPage), 3),
            0xcc => (Opcode::Cpy(Absolute), 4),

            // DEC (Decrement Memory by One)
            0xc6 => (Opcode::Dec(ZeroPage), 5),
            0xd6 => (Opcode::Dec(ZeroPageX), 6),
            0xce => (Opcode::Dec(Absolute), 6),
            0xde => (Opcode::Dec(AbsoluteX), 7),

            // DEX (Decrement Index X by One)
            0xca => (Opcode::Dex, 2),

            // DEY (Decrement Index Y by One)
            0x88 => (Opcode::Dey, 2),

            // EOR (EOR Memory with Accumulator)
            0x49 => (Opcode::Eor(Immediate), 2),
            0x45 => (Opcode::Eor(ZeroPage), 3),
            0x55 => (Opcode::Eor(ZeroPageX), 4),
            0x4d => (Opcode::Eor(Absolute), 4),
            0x5d => (Opcode::Eor(AbsoluteX), 4),
            0x59 => (Opcode::Eor(AbsoluteY), 4),
            0x41 => (Opcode::Eor(IndirectX), 6),
            0x51 => (Opcode::Eor(IndirectY), 5),

            // Memory increment instructions
            0xe6 => (Opcode::Inc(ZeroPage), 5),
            0xf6 => (Opcode::Inc(ZeroPageX), 6),
            0xee => (Opcode::Inc(Absolute), 6),
            0xfe => (Opcode::Inc(AbsoluteX), 7),
            0xe8 => (Opcode::Inx, 2),
            0xc8 => (Opcode::Iny, 2),

            // Jumps
            0x4c => (Opcode::Jmp(Absolute), 3),
            0x6c => (Opcode::Jmp(AddressingMode::Indirect), 5),

            // ORA (OR Memory with Accumulator)
            0x09 => (Opcode::Ora(Immediate), 2),
            0x05 => (Opcode::Ora(ZeroPage), 3),
            0x15 => (Opcode::Ora(ZeroPageX), 4),
            0x0d => (Opcode::Ora(Absolute), 4),
            0x1d => (Opcode::Ora(AbsoluteX), 4),
            0x19 => (Opcode::Ora(AbsoluteY), 4),
            0x01 => (Opcode::Ora(IndirectX), 6),
            0x11 => (Opcode::Ora(IndirectY), 5),

            // Stack operations
            0x48 => (Opcode::Pha, 3),
            0x08 => (Opcode::Php, 3),
            0x68 => (Opcode::Pla, 3),
            0x28 => (Opcode::Plp, 3),

            // Rotates
            0x2a => (Opcode::Rol(Accumulator), 2),
            0x26 => (Opcode::Rol(ZeroPage), 5),
            0x36 => (Opcode::Rol(ZeroPageX), 6),
            0x2e => (Opcode::Rol(Absolute), 6),
            0x3e => (Opcode::Rol(AbsoluteX), 7),
            0x6a => (Opcode::Ror(Accumulator), 2),
            0x66 => (Opcode::Ror(ZeroPage), 5),
            0x76 => (Opcode::Ror(ZeroPageX), 6),
            0x6e => (Opcode::Ror(Absolute), 6),
            0x7e => (Opcode::Ror(AbsoluteX), 7),

            // Subtract with Carry
            0xe9 => (Opcode::Sbc(Immediate), 2),
            0xe5 => (Opcode::Sbc(ZeroPage), 3),
            0xf5 => (Opcode::Sbc(ZeroPageX), 4),
            0xed => (Opcode::Sbc(Absolute), 4),
            0xfd => (Opcode::Sbc(AbsoluteX), 4),
            0xf9 => (Opcode::Sbc(AbsoluteY), 4),
            0xe1 => (Opcode::Sbc(IndirectX), 6),
            0xf1 => (Opcode::Sbc(IndirectY), 5),

            // Transfers
            0xaa => (Opcode::Tax, 2),
            0xa8 => (Opcode::Tay, 2),
            0xba => (Opcode::Tsx, 2),
            0x8a => (Opcode::Txa, 2),
            0x9a => (Opcode::Txs, 2),
            0x98 => (Opcode::Tya, 2),

            0x20 => (Opcode::Jsr, 6),
            _ => (Opcode::Nop, 1),
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.p = 0;
        self.pc = self.bus.read_u16(0xfffc);
        println!("\x1b[91mRESET           \x1b[0m {}", self);
    }

    fn step(&mut self) {
        let instruction = self.bus.read_u8(self.pc);
        let (opcode, cycles) = self.decode(instruction);
        self.pc = self.pc.wrapping_add(1);
        match opcode {
            Opcode::Nop => {}
            _ => {
                println!(
                    "{:04x} \x1b[93m{:02x} {:16}\x1b[0m {}",
                    self.pc,
                    instruction,
                    opcode.to_string().to_ascii_uppercase(),
                    self
                );
                self.execute(opcode);
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "pc:{:04x} sp:{:02x} a:{:02x} x:{:02x} y:{:02x} flags:{:08b}",
            self.pc, self.sp, self.a, self.x, self.y, self.p
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::Rom;
    use rstest::rstest;

    fn setup_cpu(prg_rom: Vec<u8>) -> Cpu {
        let rom = Rom::new_from_vec(prg_rom);
        let bus = Bus::new(rom);
        Cpu::new(bus)
    }

    fn test_program(instructions: Vec<u8>) -> Vec<u8> {
        let mut prog: Vec<u8> = instructions.to_vec();
        prog.append(&mut vec![0; 0x3ffc - instructions.len()]);
        prog.append(&mut vec![0x00, 0x80]);
        prog
    }

    #[rstest]
    #[case(vec![0x09, 0x40], 0x84, 0xc4, 0b10000000)]
    #[case(vec![0x09, 0x00], 0x00, 0x00, 0b00000010)]
    #[case(vec![0x29, 0xf0], 0x80, 0x80, 0b10000000)]
    #[case(vec![0x49, 0xf0], 0x0f, 0xff, 0b10000000)]
    fn test_accumulator_ops(
        #[case] in_prg: Vec<u8>,
        #[case] in_a: u8,
        #[case] ex_a: u8,
        #[case] ex_flags: u8,
    ) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.a = in_a;
        cpu.step();
        assert_eq!(cpu.a, ex_a);
        assert_eq!(cpu.p, ex_flags);
    }

    #[rstest]
    #[case(vec![0x10, 0x10], 0b1000_0000, 0x8001)]
    #[case(vec![0x10, 0x10], 0b0000_0000, 0x8011)]
    fn test_branches(#[case] in_prg: Vec<u8>, #[case] in_flags: u8, #[case] ex_pc: u16) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.p = in_flags;
        cpu.step();
        assert_eq!(cpu.pc, ex_pc);
    }

    #[rstest]
    #[case(vec![0x18], 0b0000_1001, 0b0000_1000)]
    #[case(vec![0xd8], 0b0000_1100, 0b0000_0100)]
    #[case(vec![0x58], 0b0000_1110, 0b0000_1010)]
    #[case(vec![0xf8], 0b0000_0110, 0b0000_1110)]
    fn test_flags(#[case] in_prg: Vec<u8>, #[case] in_flags: u8, #[case] ex_flags: u8) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.p = in_flags;
        cpu.step();
        assert_eq!(cpu.p, ex_flags);
    }

    #[rstest]
    #[case(vec![0x4C, 0x5a, 0xa5], 0xa55a)]
    #[case(vec![0x6c, 0x03, 0x80, 0x5a, 0xa5], 0xa55a)]
    fn test_jmp(#[case] in_prg: Vec<u8>, #[case] ex_pc: u16) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.step();
        assert_eq!(cpu.pc, ex_pc);
    }

    fn test_push_pull() {
        let mut cpu = setup_cpu(test_program(vec![0x48, 0x68, 0x08, 0x28]));
        cpu.reset();
        cpu.sp = 0xff;
        cpu.a = 0x12;
        cpu.step();
        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(
            cpu.bus.read_u8(0x0100 | cpu.sp.wrapping_add(1) as u16),
            0x12
        );
        cpu.a = 0x00;
        cpu.step();
        assert_eq!(cpu.a, 0x12);
        assert_eq!(cpu.sp, 0xff);
        cpu.p = 0b1010_0101;
        cpu.step();
        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(
            cpu.bus.read_u8(0x0100 | cpu.sp.wrapping_add(1) as u16),
            0b1010_0101
        );
        cpu.p = 0x00;
        cpu.step();
        assert_eq!(cpu.p, 0b1010_0101);
        assert_eq!(cpu.sp, 0xff);
    }

    #[rstest]
    #[case(vec![0xaa], 0xfa, 0, 0, 0, 0xfa, 0xfa, 0, 0)]
    #[case(vec![0xa8], 0xfa, 0, 0, 0, 0xfa, 0, 0xfa, 0)]
    #[case(vec![0xba], 0, 0, 0, 0xba, 0, 0xba, 0, 0xba)]
    #[case(vec![0x8a], 0, 0x8a, 0, 0, 0x8a, 0x8a, 0, 0)]
    #[case(vec![0x9a], 0, 0x9a, 0, 0, 0, 0x9a, 0, 0x9a)]
    #[case(vec![0x98], 0, 0, 0x98, 0, 0x98, 0, 0x98, 0)]
    fn test_transfers(
        #[case] in_prg: Vec<u8>,
        #[case] in_a: u8,
        #[case] in_x: u8,
        #[case] in_y: u8,
        #[case] in_sp: u8,
        #[case] ex_a: u8,
        #[case] ex_x: u8,
        #[case] ex_y: u8,
        #[case] ex_sp: u8,
    ) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.a = in_a;
        cpu.x = in_x;
        cpu.y = in_y;
        cpu.sp = in_sp;
        cpu.step();
        assert_eq!(cpu.a, ex_a);
        assert_eq!(cpu.x, ex_x);
        assert_eq!(cpu.y, ex_y);
        assert_eq!(cpu.sp, ex_sp);
    }

    #[rstest]
    #[case(vec![0xca], 0xff, 0, 0xfe, 0, 0b10000000)]
    #[case(vec![0x88], 0xa0, 0x05, 0xa0, 0x04, 0b00000000)]
    fn test_decs(
        #[case] in_prg: Vec<u8>,
        #[case] in_x: u8,
        #[case] in_y: u8,
        #[case] ex_x: u8,
        #[case] ex_y: u8,
        #[case] ex_flags: u8,
    ) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.x = in_x;
        cpu.y = in_y;
        cpu.step();
        assert_eq!(cpu.x, ex_x);
        assert_eq!(cpu.y, ex_y);
        assert_eq!(cpu.p, ex_flags);
    }

    #[rstest]
    #[case(vec![0x2a], 0x01, 0b00000000, 0x02, 0b00000000)]
    #[case(vec![0x2a], 0x01, 0b00000001, 0x03, 0b00000000)]
    #[case(vec![0x2a], 0x81, 0b00000000, 0x02, 0b00000001)]
    #[case(vec![0x6a], 0x80, 0b00000000, 0x40, 0b00000000)]
    #[case(vec![0x6a], 0x80, 0b00000001, 0xc0, 0b10000000)]
    #[case(vec![0x6a], 0x81, 0b00000000, 0x40, 0b00000001)]
    fn test_rotates(
        #[case] in_prg: Vec<u8>,
        #[case] in_a: u8,
        #[case] in_flags: u8,
        #[case] ex_a: u8,
        #[case] ex_flags: u8,
    ) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.a = in_a;
        cpu.p = in_flags;
        cpu.step();
        assert_eq!(cpu.a, ex_a);
        assert_eq!(cpu.p, ex_flags);
    }

    #[rstest]
    #[case(vec![0x69, 0x0f], 0xf0, 0b00000000, 0xff, 0b10000000)]
    #[case(vec![0x69, 0x0f], 0xf0, 0b00000001, 0x00, 0b00000011)]
    #[case(vec![0x69, 0x50], 0x50, 0b00000000, 0xa0, 0b11000000)]
    #[case(vec![0xe9, 0x50], 0x50, 0b00000001, 0x00, 0b00000011)]
    fn test_carries(
        #[case] in_prg: Vec<u8>,
        #[case] in_a: u8,
        #[case] in_flags: u8,
        #[case] ex_a: u8,
        #[case] ex_flags: u8,
    ) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.a = in_a;
        cpu.p = in_flags;
        cpu.step();
        assert_eq!(cpu.a, ex_a);
        assert_eq!(cpu.p, ex_flags);
    }

    // TODO add more tests
}
