#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

//https://www.masswerk.at/6502/6502_instruction_set.html
// http://www.6502.org/tutorials/6502opcodes.html

use crate::bus::Bus;
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
    Jsr(AddressingMode),
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
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
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

    fn set_flags(&mut self, value: u8) {
        if value == 0 {
            self.p |= Flag::Zero as u8
        };
        if (value & 0x80) > 0 {
            self.p |= Flag::Negative as u8
        }
    }

    fn branch(&mut self) {
        self.pc = self.get_operand(AddressingMode::Relative);
    }

    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::Nop => {}
            Opcode::Brk => {}

            Opcode::And(mode) => {
                self.a = self.a & (self.get_operand(mode) as u8);
                self.set_flags(self.a)
            }
            Opcode::Asl(mode) => {}
            Opcode::Jsr(mode) => {}
            Opcode::Bit(mode) => {}
            Opcode::Bcc => {
                if (self.p & Flag::Carry as u8) == 0 {
                    self.branch();
                }
            }
            Opcode::Bcs => {
                if (self.p & Flag::Carry as u8) != 0 {
                    self.branch();
                }
            }
            Opcode::Bvc => {
                if (self.p & Flag::Overflow as u8) == 0 {
                    self.branch();
                }
            }
            Opcode::Bvs => {
                if (self.p & Flag::Overflow as u8) != 0 {
                    self.branch();
                }
            }
            Opcode::Bne => {
                if (self.p & Flag::Zero as u8) == 0 {
                    self.branch();
                }
            }
            Opcode::Beq => {
                if (self.p & Flag::Zero as u8) != 0 {
                    self.branch();
                }
            }
            Opcode::Bpl => {
                if (self.p & Flag::Negative as u8) == 0 {
                    self.branch();
                }
            }
            Opcode::Bmi => {
                if (self.p & Flag::Negative as u8) != 0 {
                    self.branch();
                }
            }

            Opcode::Clc => {
                self.p &= !(Flag::Carry as u8);
            }
            Opcode::Sec => {
                self.p |= Flag::Carry as u8;
            }
            Opcode::Cli => {
                self.p &= !(Flag::IntDisable as u8);
            }
            Opcode::Sei => {
                self.p |= Flag::IntDisable as u8;
            }
            Opcode::Clv => {
                self.p &= !(Flag::Overflow as u8);
            }
            Opcode::Cld => {
                self.p &= !(Flag::Decimal as u8);
            }
            Opcode::Sed => {
                self.p |= Flag::Decimal as u8;
            }
            Opcode::Eor(mode) => {
                self.a = self.a ^ (self.get_operand(mode) as u8);
                self.set_flags(self.a);
            }
            Opcode::Ora(mode) => {
                self.a = self.a | (self.get_operand(mode) as u8);
                self.set_flags(self.a);
            }
            Opcode::Pha => {
                self.bus.write_u8(STACK_BYTE_HIGH | self.sp as u16, self.a);
                self.sp = self.sp.wrapping_sub(1);
            }
            Opcode::Php => {
                self.bus.write_u8(STACK_BYTE_HIGH | self.sp as u16, self.p);
                self.sp = self.sp.wrapping_sub(1);
            }
            Opcode::Pla => {
                self.sp = self.sp.wrapping_add(1);
                self.a = self.bus.read_u8(STACK_BYTE_HIGH | self.sp as u16);
                self.set_flags(self.a);
            }
            Opcode::Plp => {
                self.sp = self.sp.wrapping_add(1);
                self.p = self.bus.read_u8(STACK_BYTE_HIGH | self.sp as u16);
            }

            Opcode::Tax => {
                self.x = self.a;
                self.set_flags(self.x);
            }
            Opcode::Tay => {
                self.y = self.a;
                self.set_flags(self.y);
            }
            Opcode::Tsx => {
                self.x = self.sp;
                self.set_flags(self.x);
            }
            Opcode::Txa => {
                self.a = self.x;
                self.set_flags(self.a);
            }
            Opcode::Txs => {
                self.sp = self.x;
                self.set_flags(self.sp);
            }
            Opcode::Tya => {
                self.a = self.y;
                self.set_flags(self.a);
            }

            Opcode::Jmp(mode) => {
                self.pc = self.get_operand(mode);
            }
            _ => {
                panic!("unimplemented opcode: {}", opcode);
            }
        }
    }

    /// Given the current state of the pc and addressing mode, this function will return the appropriate operand
    /// reference: https://www.nesdev.org/wiki/CPU_addressing_modes
    fn get_operand(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.bus.read_u16(self.pc),
            AddressingMode::ZeroPage => {
                self.bus.read_u16(0x0000 + self.bus.read_u8(self.pc) as u16)
            }
            AddressingMode::ZeroPageX => self
                .bus
                .read_u16(0x0000 + self.bus.read_u8(self.pc).wrapping_add(self.x) as u16),
            AddressingMode::ZeroPageY => self
                .bus
                .read_u16(0x0000 + self.bus.read_u8(self.pc).wrapping_add(self.y) as u16),
            AddressingMode::Absolute => self.bus.read_u16(self.pc),
            AddressingMode::AbsoluteX => self.bus.read_u16(self.pc).wrapping_add(self.x as u16),
            AddressingMode::AbsoluteY => self.bus.read_u16(self.pc).wrapping_add(self.y as u16),
            AddressingMode::Relative => {
                let offset = self.bus.read_u8(self.pc) as i8;
                if offset > 0 {
                    self.pc + (offset as u16)
                } else {
                    self.pc - (offset as u16)
                }
            }
            AddressingMode::Accumulator => self.a as u16,
            AddressingMode::Indirect => self.bus.read_u16(self.bus.read_u16(self.pc)),
            AddressingMode::IndirectX => self
                .bus
                .read_u16(self.bus.read_u16(self.pc).wrapping_add(self.x as u16)),
            AddressingMode::IndirectY => self
                .bus
                .read_u16(self.bus.read_u16(self.pc).wrapping_add(self.y as u16)),
        }
    }

    /// decode takes in an opcode and outputs a tuple with opcode/addressing mode and cycles
    /// reference: https://www.nesdev.org/obelisk-6502-guide/reference.html
    fn decode(&self, opcode: u8) -> (Opcode, u8) {
        match opcode {
            // ADC (Add Memory to Accumulator with Carry)
            0x69 => (Opcode::Adc(AddressingMode::Immediate), 2),
            0x65 => (Opcode::Adc(AddressingMode::ZeroPage), 3),
            0x75 => (Opcode::Adc(AddressingMode::ZeroPageX), 4),
            0x6d => (Opcode::Adc(AddressingMode::Absolute), 4),
            0x7d => (Opcode::Adc(AddressingMode::AbsoluteX), 4),
            0x79 => (Opcode::Adc(AddressingMode::AbsoluteY), 4),
            0x61 => (Opcode::Adc(AddressingMode::IndirectX), 6),
            0x71 => (Opcode::Adc(AddressingMode::IndirectY), 5),

            // AND (bitwise AND with accumulator)
            0x29 => (Opcode::And(AddressingMode::Immediate), 2),
            0x25 => (Opcode::And(AddressingMode::ZeroPage), 3),
            0x35 => (Opcode::And(AddressingMode::ZeroPageX), 4),
            0x2d => (Opcode::And(AddressingMode::Absolute), 4),
            0x3d => (Opcode::And(AddressingMode::AbsoluteX), 4),
            0x39 => (Opcode::And(AddressingMode::AbsoluteY), 4),
            0x21 => (Opcode::And(AddressingMode::IndirectX), 6),
            0x31 => (Opcode::And(AddressingMode::IndirectY), 5),

            // ASL (Arithmetic Shift Left)
            0x0a => (Opcode::Asl(AddressingMode::Accumulator), 2),
            0x06 => (Opcode::Asl(AddressingMode::ZeroPage), 5),
            0x16 => (Opcode::Asl(AddressingMode::ZeroPageX), 6),
            0x0e => (Opcode::Asl(AddressingMode::Absolute), 2),
            0x1e => (Opcode::Asl(AddressingMode::AbsoluteX), 4),

            // Branch instructions
            0x90 => (Opcode::Bcc, 2),
            0xb0 => (Opcode::Bcs, 2),
            0xf0 => (Opcode::Beq, 2),
            0x30 => (Opcode::Bmi, 2),
            0xd0 => (Opcode::Bne, 2),
            0x10 => (Opcode::Bpl, 2),

            // BIT (test BITs in Memory With Accumulator)
            0x24 => (Opcode::Bit(AddressingMode::ZeroPage), 3),
            0x2c => (Opcode::Bit(AddressingMode::Absolute), 4),

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
            0xc9 => (Opcode::Cmp(AddressingMode::Immediate), 2),
            0xc5 => (Opcode::Cmp(AddressingMode::ZeroPage), 3),
            0xd5 => (Opcode::Cmp(AddressingMode::ZeroPageX), 4),
            0xcd => (Opcode::Cmp(AddressingMode::Absolute), 4),
            0xdd => (Opcode::Cmp(AddressingMode::AbsoluteX), 4),
            0xd9 => (Opcode::Cmp(AddressingMode::AbsoluteY), 4),
            0xc1 => (Opcode::Cmp(AddressingMode::IndirectX), 6),
            0xd1 => (Opcode::Cmp(AddressingMode::IndirectY), 5),

            // CPX (Compare Memory and Index X)
            0xe0 => (Opcode::Cpx(AddressingMode::Immediate), 2),
            0xe4 => (Opcode::Cpx(AddressingMode::ZeroPage), 3),
            0xec => (Opcode::Cpx(AddressingMode::Absolute), 4),

            // CPY (Compare Memory and Index Y)
            0xc0 => (Opcode::Cpy(AddressingMode::Immediate), 2),
            0xc4 => (Opcode::Cpy(AddressingMode::ZeroPage), 3),
            0xcc => (Opcode::Cpy(AddressingMode::Absolute), 4),

            // DEC (Decrement Memory by One)
            0xc6 => (Opcode::Dec(AddressingMode::ZeroPage), 5),
            0xd6 => (Opcode::Dec(AddressingMode::ZeroPageX), 6),
            0xce => (Opcode::Dec(AddressingMode::Absolute), 6),
            0xde => (Opcode::Dec(AddressingMode::AbsoluteX), 7),

            // DEX (Decrement Index X by One)
            0xca => (Opcode::Dex, 2),

            // DEY (Decrement Index Y by One)
            0x88 => (Opcode::Dey, 2),

            // EOR (EOR Memory with Accumulator)
            0x49 => (Opcode::Eor(AddressingMode::Immediate), 2),
            0x45 => (Opcode::Eor(AddressingMode::ZeroPage), 3),
            0x55 => (Opcode::Eor(AddressingMode::ZeroPageX), 4),
            0x4d => (Opcode::Eor(AddressingMode::Absolute), 4),
            0x5d => (Opcode::Eor(AddressingMode::AbsoluteX), 4),
            0x59 => (Opcode::Eor(AddressingMode::AbsoluteY), 4),
            0x41 => (Opcode::Eor(AddressingMode::IndirectX), 6),
            0x51 => (Opcode::Eor(AddressingMode::IndirectY), 5),

            // Memory increment instructions
            0xe6 => (Opcode::Inc(AddressingMode::ZeroPage), 5),
            0xf6 => (Opcode::Inc(AddressingMode::ZeroPageX), 6),
            0xee => (Opcode::Inc(AddressingMode::Absolute), 6),
            0xfe => (Opcode::Inc(AddressingMode::AbsoluteX), 7),
            0xe8 => (Opcode::Inx, 2),
            0xc8 => (Opcode::Iny, 2),

            // Jumps
            0x4c => (Opcode::Jmp(AddressingMode::Absolute), 3),
            0x6c => (Opcode::Jmp(AddressingMode::Indirect), 5),

            // ORA (OR Memory with Accumulator)
            0x09 => (Opcode::Ora(AddressingMode::Immediate), 2),
            0x05 => (Opcode::Ora(AddressingMode::ZeroPage), 3),
            0x15 => (Opcode::Ora(AddressingMode::ZeroPageX), 4),
            0x0d => (Opcode::Ora(AddressingMode::Absolute), 4),
            0x1d => (Opcode::Ora(AddressingMode::AbsoluteX), 4),
            0x19 => (Opcode::Ora(AddressingMode::AbsoluteY), 4),
            0x01 => (Opcode::Ora(AddressingMode::IndirectX), 6),
            0x11 => (Opcode::Ora(AddressingMode::IndirectY), 5),

            // Stack operations
            0x48 => (Opcode::Pha, 3),
            0x08 => (Opcode::Php, 3),
            0x68 => (Opcode::Pla, 3),
            0x28 => (Opcode::Plp, 3),

            // Transfers
            0xaa => (Opcode::Tax, 2),
            0xa8 => (Opcode::Tay, 2),
            0xba => (Opcode::Tsx, 2),
            0x8a => (Opcode::Txa, 2),
            0x9a => (Opcode::Txs, 2),
            0x98 => (Opcode::Tya, 2),

            0x20 => (Opcode::Jsr(AddressingMode::Absolute), 6),
            _ => (Opcode::Nop, 1),
        }
    }

    fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.p = 0;
        self.pc = self.bus.read_u16(0xfffc);
        println!("RESET -> {}", self)
    }

    fn step(&mut self) {
        let instruction = self.bus.read_u8(self.pc);
        let (opcode, cycles) = self.decode(instruction);
        self.pc = self.pc.wrapping_add(1);
        println!(
            "{:02x} {}",
            instruction,
            opcode.to_string().to_ascii_uppercase()
        );
        self.execute(opcode);
        println!("{}", self)
    }

    fn run(&mut self) {
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
        let mut bus = Bus::new();
        bus.load(rom);
        Cpu::new(bus)
    }

    fn test_program(instructions: Vec<u8>) -> Vec<u8> {
        let mut prog: Vec<u8> = instructions.to_vec();
        prog.append(&mut vec![0; 0xbfdc - instructions.len()]);
        prog.append(&mut vec![0x20, 0x40]);
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
    #[case(vec![0x10, 0x10], 0b1000_0000, 0x4021)]
    #[case(vec![0x10, 0x10], 0b0000_0000, 0x4031)]
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
    #[case(vec![0x6c, 0x23, 0x40, 0x5a, 0xa5], 0xa55a)]
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

    // TODO add more tests
}
