#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// https://www.masswerk.at/6502/6502_instruction_set.html
// http://www.6502.org/tutorials/6502opcodes.html

use crate::bus::Bus;
use crate::cpu::AddressingMode::{
    Absolute, AbsoluteX, AbsoluteY, Accumulator, Immediate, Implied, Indirect, IndirectX,
    IndirectY, Relative, ZeroPage, ZeroPageX, ZeroPageY,
};
use crate::cpu::Flag::Zero;
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
    cycles: u64,
    bus: Bus,
}

#[derive(Debug)]
enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    Immediate,
    Implied,
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
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
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
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
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

struct Instruction {
    opcode: Opcode,
    mode: AddressingMode,
    length: u8,
    cycles: u8,
}

struct InstructionBytes<'a> {
    instruction: &'a Instruction,
    bytes: Vec<u8>,
}

impl Display for InstructionBytes<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let instruction_string = match self.instruction.mode {
            Absolute => {
                format!(
                    "${:04X?}",
                    u16::from_le_bytes([self.bytes[1], self.bytes[2]])
                )
            }
            AbsoluteX => {
                format!(
                    "${:04X?},X",
                    u16::from_le_bytes([self.bytes[1], self.bytes[2]])
                )
            }
            AbsoluteY => {
                format!(
                    "${:04X?},Y",
                    u16::from_le_bytes([self.bytes[1], self.bytes[2]])
                )
            }
            Immediate => {
                format!("#${:02X?}", &self.bytes[1])
            }
            Indirect => {
                format!(
                    "(${:04X?})",
                    u16::from_le_bytes([self.bytes[1], self.bytes[2]])
                )
            }
            IndirectX => {
                format!(
                    "(${:04X?},X)",
                    u16::from_le_bytes([self.bytes[1], self.bytes[2]])
                )
            }
            IndirectY => {
                format!(
                    "(${:04X?}),Y",
                    u16::from_le_bytes([self.bytes[1], self.bytes[2]])
                )
            }
            ZeroPage => {
                format!("${:02X?}", &self.bytes[1])
            }
            ZeroPageX => {
                format!("${:02X?},X", &self.bytes[1])
            }
            ZeroPageY => {
                format!("${:02X?},Y", &self.bytes[1])
            }
            _ => format!(""),
        };
        let mut bytes_string = String::new();
        for b in &self.bytes {
            bytes_string.push_str(format!("{:02X} ", b).as_str())
        }
        write!(
            f,
            "{:<9} {:<4} {:<8}",
            bytes_string.replace("[", "").replace("]", ""),
            self.instruction.opcode.to_string().to_ascii_uppercase(),
            instruction_string
                .replace("[", "")
                .replace("]", "")
                .replace(",", "")
        )?;
        Ok(())
    }
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
            cycles: 0,
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

    fn branch(&mut self, condition: bool) -> u16 {
        if condition {
            self.get_operand(Relative)
        } else {
            self.pc
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

    fn execute(&mut self, instruction: Instruction) {
        let mut new_pc = self.pc.wrapping_add((instruction.length - 1) as u16);
        match instruction.opcode {
            Opcode::Nop => {}

            // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
            Opcode::Adc => {
                let carry: u8 = if self.is_flag_set(Flag::Carry) { 1 } else { 0 };
                let operand = self.get_operand(instruction.mode) as u8;
                let result = self.a.wrapping_add(operand.wrapping_add(carry));
                self.change_flag(
                    Flag::Overflow,
                    (operand ^ result) & (self.a ^ result) & 0x80 != 0,
                );
                self.a = result;
                self.set_zero_negative_flags(self.a);
            }

            Opcode::And => {
                self.a = self.a & (self.get_operand(instruction.mode) as u8);
                self.set_zero_negative_flags(self.a)
            }
            Opcode::Asl => {
                let op = self.get_operand(instruction.mode);
                self.a = op.rotate_left(1) as u8 & 0xfe;
                if op & 0x80 != 0 {
                    self.set_flag(Flag::Carry);
                } else {
                    self.clear_flag(Flag::Carry);
                }
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Bcc => {
                new_pc = self.branch(self.is_flag_clear(Flag::Carry));
            }
            Opcode::Bcs => {
                new_pc = self.branch(self.is_flag_set(Flag::Carry));
            }
            Opcode::Bvc => {
                new_pc = self.branch(self.is_flag_clear(Flag::Overflow));
            }
            Opcode::Bvs => {
                new_pc = self.branch(self.is_flag_set(Flag::Overflow));
            }
            Opcode::Bne => {
                new_pc = self.branch(self.is_flag_clear(Flag::Zero));
            }
            Opcode::Beq => {
                new_pc = self.branch(self.is_flag_set(Flag::Zero));
            }
            Opcode::Bpl => {
                new_pc = self.branch(self.is_flag_clear(Flag::Negative));
            }
            Opcode::Bmi => {
                new_pc = self.branch(self.is_flag_set(Flag::Negative));
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
            Opcode::Inc => {
                let addr = self.get_operand_address(instruction.mode);
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
            Opcode::Bit => {
                let result = self.a & self.get_operand(instruction.mode) as u8;
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
                let tgt_addr = self.get_operand(Absolute);
                let ret_addr = self.pc - 1;
                self.stack_push_u16(ret_addr);
                println!("jsr target addr {:04X}", tgt_addr);
                new_pc = tgt_addr;
                // TODO write tests
            }
            Opcode::Cmp => {
                self.compare(instruction.mode, self.a);
                // TODO write tests
            }
            Opcode::Cpx => {
                self.compare(instruction.mode, self.x);
                // TODO write tests
            }
            Opcode::Cpy => {
                self.compare(instruction.mode, self.y);
                // TODO write tests
            }
            Opcode::Dec => {
                let addr = self.get_operand_address(instruction.mode);
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
            Opcode::Eor => {
                self.a = self.a ^ (self.get_operand(instruction.mode) as u8);
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Ora => {
                self.a = self.a | (self.get_operand(instruction.mode) as u8);
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
            Opcode::Rol => {
                let carry_in = if self.p & (Flag::Carry as u8) != 0 {
                    0x01
                } else {
                    0x00
                };
                let op = self.get_operand(instruction.mode);
                if op & 0x80 != 0 {
                    self.set_flag(Flag::Carry);
                } else {
                    self.clear_flag(Flag::Carry);
                }
                self.a = op.rotate_left(1) as u8 | carry_in;
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Ror => {
                let carry_in = if self.p & (Flag::Carry as u8) != 0 {
                    0x80
                } else {
                    0x00
                };
                let op = self.get_operand(instruction.mode);
                if op & 0x01 != 0 {
                    self.set_flag(Flag::Carry);
                } else {
                    self.clear_flag(Flag::Carry);
                }
                self.a = op.rotate_right(1) as u8 | carry_in;
                self.set_zero_negative_flags(self.a);
            }

            Opcode::Sbc => {
                let carry: u8 = if self.is_flag_clear(Flag::Carry) {
                    1
                } else {
                    0
                };
                let operand = self.get_operand(instruction.mode) as u8;
                let result = self.a.wrapping_sub(operand.wrapping_add(carry));
                self.change_flag(
                    Flag::Overflow,
                    (operand ^ result) & (self.a ^ result) & 0x80 != 0,
                );
                self.a = result;
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Sta => {
                let addr = self.get_operand_address(instruction.mode);
                self.bus.write_u8(addr, self.a);
            }

            Opcode::Stx => {
                let addr = self.get_operand_address(instruction.mode);
                self.bus.write_u8(addr, self.x);
            }

            Opcode::Sty => {
                let addr = self.get_operand_address(instruction.mode);
                self.bus.write_u8(addr, self.y);
            }

            Opcode::Lda => {
                self.a = self.get_operand(instruction.mode) as u8;
            }

            Opcode::Ldx => {
                self.x = self.get_operand(instruction.mode) as u8;
            }

            Opcode::Ldy => {
                self.y = self.get_operand(instruction.mode) as u8;
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

            Opcode::Jmp => {
                new_pc = self.get_operand(instruction.mode);
                println!("jmp! pc is now {:02X}", self.pc)
            }
            _ => {
                panic!("unimplemented opcode: {}", instruction.opcode);
            }
        }

        self.pc = new_pc;
    }

    /// Returns the address of the operand given the addressing mode. Some mods such as Immediate and Accumulator will return 0 as an invalid state
    fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            Immediate => self.pc,
            ZeroPage => 0x0000 + self.bus.read_u8(self.pc) as u16,
            ZeroPageX => 0x0000 + self.bus.read_u8(self.pc).wrapping_add(self.x) as u16,
            ZeroPageY => 0x0000 + self.bus.read_u8(self.pc).wrapping_add(self.y) as u16,
            Absolute => {
                println!("absolute pc {:02X}", self.pc);
                self.pc
            }
            AbsoluteX => self.pc.wrapping_add(self.x as u16),
            AbsoluteY => self.pc.wrapping_add(self.y as u16),

            Indirect => self.bus.read_u16(self.pc),
            IndirectX => self.bus.read_u16(self.pc).wrapping_add(self.x as u16),
            IndirectY => self.bus.read_u16(self.pc).wrapping_add(self.y as u16),
            _ => 0,
        }
    }

    /// Given the current state of the pc and addressing mode, this function will return the
    /// appropriate operand.
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

    /// decode takes in an opcode and outputs an instruction structure
    /// reference: https://www.nesdev.org/obelisk-6502-guide/reference.html
    fn decode(&self, opcode: u8) -> Instruction {
        match opcode {
            // ADC (Add Memory to Accumulator with Carry)
            0x69 => Instruction {
                opcode: Opcode::Adc,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0x65 => Instruction {
                opcode: Opcode::Adc,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x75 => Instruction {
                opcode: Opcode::Adc,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0x6d => Instruction {
                opcode: Opcode::Adc,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0x7d => Instruction {
                opcode: Opcode::Adc,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },
            0x79 => Instruction {
                opcode: Opcode::Adc,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },
            0x61 => Instruction {
                opcode: Opcode::Adc,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0x71 => Instruction {
                opcode: Opcode::Adc,
                mode: IndirectY,
                length: 2,
                cycles: 5,
            },

            // AND (bitwise AND with accumulator)
            0x29 => Instruction {
                opcode: Opcode::And,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0x25 => Instruction {
                opcode: Opcode::And,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x35 => Instruction {
                opcode: Opcode::And,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0x2d => Instruction {
                opcode: Opcode::And,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0x3d => Instruction {
                opcode: Opcode::And,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },
            0x39 => Instruction {
                opcode: Opcode::And,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },
            0x21 => Instruction {
                opcode: Opcode::And,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0x31 => Instruction {
                opcode: Opcode::And,
                mode: IndirectY,
                length: 2,
                cycles: 5,
            },

            // ASL (Arithmetic Shift Left)
            0x0a => Instruction {
                opcode: Opcode::Asl,
                mode: Accumulator,
                length: 1,
                cycles: 2,
            },
            0x06 => Instruction {
                opcode: Opcode::Asl,
                mode: ZeroPage,
                length: 2,
                cycles: 5,
            },
            0x16 => Instruction {
                opcode: Opcode::Asl,
                mode: ZeroPageX,
                length: 2,
                cycles: 6,
            },
            0x0e => Instruction {
                opcode: Opcode::Asl,
                mode: Absolute,
                length: 3,
                cycles: 2,
            },
            0x1e => Instruction {
                opcode: Opcode::Asl,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },

            // Branch instructions
            0x90 => Instruction {
                opcode: Opcode::Bcc,
                mode: Implied,
                length: 2,
                cycles: 2,
            },
            0xb0 => Instruction {
                opcode: Opcode::Bcs,
                mode: Relative,
                length: 2,
                cycles: 2,
            },
            0xf0 => Instruction {
                opcode: Opcode::Beq,
                mode: Relative,
                length: 2,
                cycles: 2,
            },
            0x30 => Instruction {
                opcode: Opcode::Bmi,
                mode: Relative,
                length: 2,
                cycles: 2,
            },
            0xd0 => Instruction {
                opcode: Opcode::Bne,
                mode: Relative,
                length: 2,
                cycles: 2,
            },
            0x10 => Instruction {
                opcode: Opcode::Bpl,
                mode: Relative,
                length: 2,
                cycles: 2,
            },

            // BIT (test BITs in Memory With Accumulator)
            0x24 => Instruction {
                opcode: Opcode::Bit,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x2c => Instruction {
                opcode: Opcode::Bit,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },

            // BRK (Force break)
            0x00 => Instruction {
                opcode: Opcode::Brk,
                mode: Implied,
                length: 1,
                cycles: 7,
            },

            // Flag operations
            0x18 => Instruction {
                opcode: Opcode::Clc,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0x38 => Instruction {
                opcode: Opcode::Sec,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0x58 => Instruction {
                opcode: Opcode::Cli,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0x78 => Instruction {
                opcode: Opcode::Sei,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0xb8 => Instruction {
                opcode: Opcode::Clv,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0xd8 => Instruction {
                opcode: Opcode::Cld,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0xf8 => Instruction {
                opcode: Opcode::Sed,
                mode: Implied,
                length: 1,
                cycles: 2,
            },

            // CMP (Compare Memory with Accumulator)
            0xc9 => Instruction {
                opcode: Opcode::Cmp,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0xc5 => Instruction {
                opcode: Opcode::Cmp,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0xd5 => Instruction {
                opcode: Opcode::Cmp,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0xcd => Instruction {
                opcode: Opcode::Cmp,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0xdd => Instruction {
                opcode: Opcode::Cmp,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },
            0xd9 => Instruction {
                opcode: Opcode::Cmp,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },
            0xc1 => Instruction {
                opcode: Opcode::Cmp,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0xd1 => Instruction {
                opcode: Opcode::Cmp,
                mode: IndirectY,
                length: 2,
                cycles: 5,
            },

            // CPX (Compare Memory and Index X)
            0xe0 => Instruction {
                opcode: Opcode::Cpx,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0xe4 => Instruction {
                opcode: Opcode::Cpx,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0xec => Instruction {
                opcode: Opcode::Cpx,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },

            // CPY (Compare Memory and Index Y)
            0xc0 => Instruction {
                opcode: Opcode::Cpy,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0xc4 => Instruction {
                opcode: Opcode::Cpy,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0xcc => Instruction {
                opcode: Opcode::Cpy,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },

            // DEC (Decrement Memory by One)
            0xc6 => Instruction {
                opcode: Opcode::Dec,
                mode: ZeroPage,
                length: 2,
                cycles: 5,
            },
            0xd6 => Instruction {
                opcode: Opcode::Dec,
                mode: ZeroPageX,
                length: 2,
                cycles: 6,
            },
            0xce => Instruction {
                opcode: Opcode::Dec,
                mode: Absolute,
                length: 3,
                cycles: 6,
            },
            0xde => Instruction {
                opcode: Opcode::Dec,
                mode: AbsoluteX,
                length: 3,
                cycles: 7,
            },

            // DEX (Decrement Index X by One)
            0xca => Instruction {
                opcode: Opcode::Dex,
                mode: Implied,
                length: 1,
                cycles: 2,
            },

            // DEY (Decrement Index Y by One)
            0x88 => Instruction {
                opcode: Opcode::Dey,
                mode: Implied,
                length: 1,
                cycles: 2,
            },

            // EOR (EOR Memory with Accumulator)
            0x49 => Instruction {
                opcode: Opcode::Eor,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0x45 => Instruction {
                opcode: Opcode::Eor,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x55 => Instruction {
                opcode: Opcode::Eor,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0x4d => Instruction {
                opcode: Opcode::Eor,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0x5d => Instruction {
                opcode: Opcode::Eor,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },
            0x59 => Instruction {
                opcode: Opcode::Eor,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },
            0x41 => Instruction {
                opcode: Opcode::Eor,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0x51 => Instruction {
                opcode: Opcode::Eor,
                mode: IndirectY,
                length: 2,
                cycles: 5,
            },

            // Memory increment instructions
            0xe6 => Instruction {
                opcode: Opcode::Inc,
                mode: ZeroPage,
                length: 2,
                cycles: 5,
            },
            0xf6 => Instruction {
                opcode: Opcode::Inc,
                mode: ZeroPageX,
                length: 2,
                cycles: 6,
            },
            0xee => Instruction {
                opcode: Opcode::Inc,
                mode: Absolute,
                length: 3,
                cycles: 6,
            },
            0xfe => Instruction {
                opcode: Opcode::Inc,
                mode: AbsoluteX,
                length: 3,
                cycles: 7,
            },
            0xe8 => Instruction {
                opcode: Opcode::Inx,
                mode: Indirect,
                length: 1,
                cycles: 2,
            },
            0xc8 => Instruction {
                opcode: Opcode::Iny,
                mode: Indirect,
                length: 1,
                cycles: 2,
            },

            // Jumps
            0x4c => Instruction {
                opcode: Opcode::Jmp,
                mode: Absolute,
                length: 3,
                cycles: 3,
            },
            0x6c => Instruction {
                opcode: Opcode::Jmp,
                mode: Indirect,
                length: 3,
                cycles: 5,
            },

            // LDA (load Accumulator)
            0xA9 => Instruction {
                opcode: Opcode::Lda,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0xA5 => Instruction {
                opcode: Opcode::Lda,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0xB5 => Instruction {
                opcode: Opcode::Lda,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0xAD => Instruction {
                opcode: Opcode::Lda,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0xBD => Instruction {
                opcode: Opcode::Lda,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },
            0xB9 => Instruction {
                opcode: Opcode::Lda,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },
            0xA1 => Instruction {
                opcode: Opcode::Lda,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0xB1 => Instruction {
                opcode: Opcode::Lda,
                mode: IndirectY,
                length: 2,
                cycles: 5,
            },

            // LDX (Load X Register)
            0xA2 => Instruction {
                opcode: Opcode::Ldx,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0xA6 => Instruction {
                opcode: Opcode::Ldx,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0xB6 => Instruction {
                opcode: Opcode::Ldx,
                mode: ZeroPageY,
                length: 2,
                cycles: 4,
            },
            0xAE => Instruction {
                opcode: Opcode::Ldx,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0xBE => Instruction {
                opcode: Opcode::Ldx,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },

            // LDY (Load Y Register)
            0xA0 => Instruction {
                opcode: Opcode::Ldy,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0xA4 => Instruction {
                opcode: Opcode::Ldy,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0xB4 => Instruction {
                opcode: Opcode::Ldy,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0xAC => Instruction {
                opcode: Opcode::Ldy,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0xBC => Instruction {
                opcode: Opcode::Ldy,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },

            // ORA (OR Memory with Accumulator
            0x09 => Instruction {
                opcode: Opcode::Ora,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0x05 => Instruction {
                opcode: Opcode::Ora,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x15 => Instruction {
                opcode: Opcode::Ora,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0x0d => Instruction {
                opcode: Opcode::Ora,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0x1d => Instruction {
                opcode: Opcode::Ora,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },
            0x19 => Instruction {
                opcode: Opcode::Ora,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },
            0x01 => Instruction {
                opcode: Opcode::Ora,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0x11 => Instruction {
                opcode: Opcode::Ora,
                mode: IndirectY,
                length: 2,
                cycles: 5,
            },

            // Stack operations
            0x48 => Instruction {
                opcode: Opcode::Pha,
                mode: Implied,
                length: 1,
                cycles: 3,
            },
            0x08 => Instruction {
                opcode: Opcode::Php,
                mode: Implied,
                length: 1,
                cycles: 3,
            },
            0x68 => Instruction {
                opcode: Opcode::Pla,
                mode: Implied,
                length: 1,
                cycles: 3,
            },
            0x28 => Instruction {
                opcode: Opcode::Plp,
                mode: Implied,
                length: 1,
                cycles: 3,
            },

            // Rotates
            0x2a => Instruction {
                opcode: Opcode::Rol,
                mode: Accumulator,
                length: 1,
                cycles: 2,
            },
            0x26 => Instruction {
                opcode: Opcode::Rol,
                mode: ZeroPage,
                length: 2,
                cycles: 5,
            },
            0x36 => Instruction {
                opcode: Opcode::Rol,
                mode: ZeroPageX,
                length: 2,
                cycles: 6,
            },
            0x2e => Instruction {
                opcode: Opcode::Rol,
                mode: Absolute,
                length: 3,
                cycles: 6,
            },
            0x3e => Instruction {
                opcode: Opcode::Rol,
                mode: AbsoluteX,
                length: 3,
                cycles: 7,
            },
            0x6a => Instruction {
                opcode: Opcode::Ror,
                mode: Accumulator,
                length: 1,
                cycles: 2,
            },
            0x66 => Instruction {
                opcode: Opcode::Ror,
                mode: ZeroPage,
                length: 2,
                cycles: 5,
            },
            0x76 => Instruction {
                opcode: Opcode::Ror,
                mode: ZeroPageX,
                length: 2,
                cycles: 6,
            },
            0x6e => Instruction {
                opcode: Opcode::Ror,
                mode: Absolute,
                length: 3,
                cycles: 6,
            },
            0x7e => Instruction {
                opcode: Opcode::Ror,
                mode: AbsoluteX,
                length: 3,
                cycles: 7,
            },

            // Subtract with Carry
            0xe9 => Instruction {
                opcode: Opcode::Sbc,
                mode: Immediate,
                length: 2,
                cycles: 2,
            },
            0xe5 => Instruction {
                opcode: Opcode::Sbc,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0xf5 => Instruction {
                opcode: Opcode::Sbc,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0xed => Instruction {
                opcode: Opcode::Sbc,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0xfd => Instruction {
                opcode: Opcode::Sbc,
                mode: AbsoluteX,
                length: 3,
                cycles: 4,
            },
            0xf9 => Instruction {
                opcode: Opcode::Sbc,
                mode: AbsoluteY,
                length: 3,
                cycles: 4,
            },
            0xe1 => Instruction {
                opcode: Opcode::Sbc,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0xf1 => Instruction {
                opcode: Opcode::Sbc,
                mode: IndirectY,
                length: 2,
                cycles: 5,
            },

            // STA (Store A register)
            0x85 => Instruction {
                opcode: Opcode::Sta,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x95 => Instruction {
                opcode: Opcode::Sta,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0x8d => Instruction {
                opcode: Opcode::Sta,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },
            0x9d => Instruction {
                opcode: Opcode::Sta,
                mode: AbsoluteX,
                length: 3,
                cycles: 5,
            },
            0x99 => Instruction {
                opcode: Opcode::Sta,
                mode: AbsoluteY,
                length: 3,
                cycles: 5,
            },
            0x81 => Instruction {
                opcode: Opcode::Sta,
                mode: IndirectX,
                length: 2,
                cycles: 6,
            },
            0x91 => Instruction {
                opcode: Opcode::Sta,
                mode: IndirectY,
                length: 2,
                cycles: 6,
            },

            // STX (Store X register)
            0x86 => Instruction {
                opcode: Opcode::Stx,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x96 => Instruction {
                opcode: Opcode::Stx,
                mode: ZeroPageY,
                length: 2,
                cycles: 4,
            },
            0x8e => Instruction {
                opcode: Opcode::Stx,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },

            // STY (Store Y register)
            0x84 => Instruction {
                opcode: Opcode::Sty,
                mode: ZeroPage,
                length: 2,
                cycles: 3,
            },
            0x94 => Instruction {
                opcode: Opcode::Sty,
                mode: ZeroPageX,
                length: 2,
                cycles: 4,
            },
            0x8c => Instruction {
                opcode: Opcode::Sty,
                mode: Absolute,
                length: 3,
                cycles: 4,
            },

            // Transfers
            0xaa => Instruction {
                opcode: Opcode::Tax,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0xa8 => Instruction {
                opcode: Opcode::Tay,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0xba => Instruction {
                opcode: Opcode::Tsx,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0x8a => Instruction {
                opcode: Opcode::Txa,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0x9a => Instruction {
                opcode: Opcode::Txs,
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0x98 => Instruction {
                opcode: Opcode::Tya,
                mode: Implied,
                length: 1,
                cycles: 2,
            },

            0x20 => Instruction {
                opcode: Opcode::Jsr,
                mode: Absolute,
                length: 3,
                cycles: 6,
            },
            _ => Instruction {
                opcode: Opcode::Nop,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.p = 0;
        self.pc = 0xc000;
        println!(
            "\x1b[94m{:04X}\x1b[0m \x1b[91m{:<23}\x1b[0m {}",
            self.pc, "RESET", self
        )
    }

    pub fn step(&mut self) {
        let instruction = self.decode(self.bus.read_u8(self.pc));
        let instruction_bytes = InstructionBytes {
            instruction: &instruction,
            bytes: self.bus.read_bytes(self.pc, instruction.length),
        };

        self.cycles = self.cycles.wrapping_add(instruction.cycles as u64);

        println!(
            "\x1b[94m{:04X}\x1b[0m \x1b[93m{}\x1b[0m {}",
            self.pc, instruction_bytes, self
        );
        self.pc = self.pc.wrapping_add(1);
        self.execute(instruction);
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "A:{:04X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.a, self.x, self.y, self.p, self.sp, self.cycles
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
    #[case(vec![0x10, 0x10], 0b1000_0000, 0x8002)]
    #[case(vec![0x10, 0x10], 0b0000_0000, 0x8012)]
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
