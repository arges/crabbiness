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
use log::debug;
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
    pub bus: Bus,
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
    Kil,
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

impl InstructionBytes<'_> {
    fn get_address(&self) -> u16 {
        assert!(self.bytes.len() == 3);
        u16::from_le_bytes([self.bytes[1], self.bytes[2]])
    }

    fn get_immediate(&self) -> u8 {
        assert!(self.bytes.len() == 2);
        self.bytes[1]
    }

    fn get_offset(&self) -> i8 {
        assert!(self.bytes.len() == 2);
        self.bytes[1] as i8
    }
}

impl Display for InstructionBytes<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let instruction_string = match self.instruction.mode {
            Absolute => {
                format!("${:04X?}", self.get_address())
            }
            AbsoluteX => {
                format!("${:04X?},X", self.get_address())
            }
            AbsoluteY => {
                format!("${:04X?},Y", self.get_address())
            }
            Immediate => {
                format!("#${:02X?}", self.get_immediate())
            }
            Indirect => {
                format!("(${:04X?})", self.get_address())
            }
            IndirectX => {
                format!("(${:02X?},X)", self.get_immediate())
            }
            IndirectY => {
                format!("(${:02X?}),Y", self.get_immediate())
            }
            ZeroPage => {
                format!("${:02X?}", self.get_immediate())
            }
            ZeroPageX => {
                format!("${:02X?},X", self.get_immediate())
            }
            ZeroPageY => {
                format!("${:02X?},Y", self.get_immediate())
            }
            Relative => {
                format!("~{:02X?}", self.get_immediate())
            }
            _ => "".to_string(),
        };
        let mut bytes_string = String::new();
        for b in &self.bytes {
            bytes_string.push_str(format!("{:02X} ", b).as_str())
        }
        // TODO fix this hack
        write!(
            f,
            "{:<9} {:<3} {:<8}",
            bytes_string.replace('[', "").replace(']', ""),
            self.instruction.opcode.to_string().to_ascii_uppercase(),
            instruction_string
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

    pub fn nmi(&mut self) -> u16 {
        debug!("executing nmi");
        self.stack_push_u16(self.pc);
        self.clear_flag(Flag::Break);
        self.set_flag(Flag::Unused);
        self.stack_push_u8(self.p);
        self.set_flag(Flag::IntDisable);
        let new_pc = self.bus.read_u16(0xfffa);
        self.pc = new_pc;
        new_pc
    }

    fn set_zero_negative_flags(&mut self, value: u8) {
        self.change_flag(Flag::Zero, value == 0);
        self.change_flag(Flag::Negative, (value & 0x80) > 0);
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

    fn branch(&mut self, b: &InstructionBytes, condition: bool) -> u16 {
        let new_pc = self.pc.wrapping_add(b.bytes.len() as u16);
        if condition {
            new_pc.wrapping_add(b.get_offset() as u16)
        } else {
            new_pc
        }
    }

    fn shift_left(&mut self, input: u8) -> (u8, bool) {
        (input >> 1, input & 1 == 1)
    }

    fn compare(&mut self, b: &InstructionBytes, reg: u8) {
        let op = self.get_operand(b) as u8;
        debug!("compare op {:02X} reg {:02X}", op, reg);
        let result = reg.wrapping_sub(op);
        self.change_flag(Flag::Carry, reg >= op);
        self.set_zero_negative_flags(result);
    }

    fn stack_push_u8(&mut self, value: u8) {
        debug!("push u8 @ {:02X} <- {:02X}", self.sp, value);
        self.bus.write_u8(STACK_BYTE_HIGH | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_push_u16(&mut self, value: u16) {
        self.stack_push_u8(((value & 0xff00) >> 8) as u8);
        self.stack_push_u8((value & 0xff) as u8);
    }

    fn stack_pop_u8(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let val = self.bus.read_u8(STACK_BYTE_HIGH | self.sp as u16);
        debug!(" pop u8 @ {:02X} -> {:02X}", self.sp, val);
        val
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lsb = self.stack_pop_u8() as u16;
        let msb = self.stack_pop_u8() as u16;
        (msb << 8) | lsb
    }

    fn add_to_a(&mut self, operand: u8) {
        let carry: u8 = if self.is_flag_set(Flag::Carry) { 1 } else { 0 };
        let result: u16 = self.a as u16 + operand as u16 + carry as u16;

        self.change_flag(Flag::Carry, result > 0xff);
        self.change_flag(
            Flag::Overflow,
            (operand ^ result as u8) & (self.a ^ result as u8) & 0x80 != 0,
        );
        self.a = result as u8;
        self.set_zero_negative_flags(self.a);
    }

    fn set_result(&mut self, b: &InstructionBytes, result: u8) {
        match b.instruction.mode {
            Accumulator => {
                self.a = result;
            }
            _ => {
                let addr = self.get_operand_address(b);
                self.bus.write_u8(addr, result);
            }
        }
    }

    fn execute(&mut self, b: &InstructionBytes) {
        let mut new_pc = self.pc.wrapping_add((b.instruction.length) as u16);
        match b.instruction.opcode {
            Opcode::Nop => {}

            // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
            Opcode::Adc => {
                let operand = self.get_operand(b) as u8;
                self.add_to_a(operand)
            }

            Opcode::And => {
                self.a &= self.get_operand(b) as u8;
                self.set_zero_negative_flags(self.a)
            }
            Opcode::Asl => {
                let op = self.get_operand(b);
                self.change_flag(Flag::Carry, op & 0x80 != 0);
                let result = op.rotate_left(1) as u8 & 0xfe;
                self.set_zero_negative_flags(result);
                match b.instruction.mode {
                    Accumulator => {
                        self.a = result;
                    }
                    _ => {
                        let addr = self.get_operand_address(b);
                        self.bus.write_u8(addr, result);
                    }
                }
            }
            Opcode::Bcc => {
                new_pc = self.branch(b, self.is_flag_clear(Flag::Carry));
            }
            Opcode::Bcs => {
                new_pc = self.branch(b, self.is_flag_set(Flag::Carry));
            }
            Opcode::Bvc => {
                new_pc = self.branch(b, self.is_flag_clear(Flag::Overflow));
            }
            Opcode::Bvs => {
                new_pc = self.branch(b, self.is_flag_set(Flag::Overflow));
            }
            Opcode::Bne => {
                new_pc = self.branch(b, self.is_flag_clear(Flag::Zero));
            }
            Opcode::Beq => {
                new_pc = self.branch(b, self.is_flag_set(Flag::Zero));
            }
            Opcode::Bpl => {
                new_pc = self.branch(b, self.is_flag_clear(Flag::Negative));
            }
            Opcode::Bmi => {
                new_pc = self.branch(b, self.is_flag_set(Flag::Negative));
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
                let addr = self.get_operand_address(b);
                let value = self.bus.read_u8(addr).wrapping_add(1);
                self.bus.write_u8(addr, value);
                self.set_zero_negative_flags(value);
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
                let operand = self.get_operand(b) as u8;
                let result = self.a & operand;
                self.change_flag(Flag::Zero, result == 0);
                self.change_flag(Flag::Negative, operand & 0x80 != 0);
                self.change_flag(Flag::Overflow, operand & 0x40 != 0);
                debug!("bit: result is {:02X} operand is {:02X}", result, operand);
            }
            Opcode::Brk => {
                new_pc = self.nmi();
            }
            Opcode::Rti => {
                self.p = self.stack_pop_u8();
                self.pc = self.stack_pop_u16();
                self.clear_flag(Flag::Break);
                self.set_flag(Flag::Unused);
                // TODO write tests
            }
            Opcode::Jsr => {
                let tgt_addr = self.get_operand_address(b);
                let ret_addr = self.pc.wrapping_add(2);
                self.stack_push_u16(ret_addr);
                debug!("jsr tgt_addr {:04X} ret_addr {:04X}", tgt_addr, ret_addr);
                new_pc = tgt_addr;
                // TODO write tests
            }
            Opcode::Rts => {
                new_pc = self.stack_pop_u16().wrapping_add(1);
                debug!("rts new_pc {:04X}", new_pc);
                // TODO write tests
            }

            Opcode::Cmp => {
                self.compare(b, self.a);
                // TODO write tests
            }
            Opcode::Cpx => {
                self.compare(b, self.x);
                // TODO write tests
            }
            Opcode::Cpy => {
                self.compare(b, self.y);
                // TODO write tests
            }
            Opcode::Dec => {
                let addr = self.get_operand_address(b);
                let value = self.bus.read_u8(addr).wrapping_sub(1);
                self.bus.write_u8(addr, value);
                self.set_zero_negative_flags(value);
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
                self.a ^= self.get_operand(b) as u8;
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Ora => {
                self.a |= self.get_operand(b) as u8;
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Pha => {
                self.stack_push_u8(self.a);
            }
            Opcode::Php => {
                self.stack_push_u8(self.p | Flag::Break as u8 | Flag::Unused as u8);
            }
            Opcode::Pla => {
                self.a = self.stack_pop_u8();
                self.set_zero_negative_flags(self.a);
            }
            Opcode::Plp => {
                self.p = self.stack_pop_u8();
                self.clear_flag(Flag::Break);
                self.set_flag(Flag::Unused)
            }
            Opcode::Rol => {
                let carry_in = if self.p & (Flag::Carry as u8) != 0 {
                    0x01
                } else {
                    0x00
                };
                let op = self.get_operand(b);
                self.change_flag(Flag::Carry, op & 0x80 != 0);
                let result = op.rotate_left(1) as u8 | carry_in;
                self.set_zero_negative_flags(result);
                self.set_result(b, result);
            }
            Opcode::Ror => {
                let carry_in = if self.p & (Flag::Carry as u8) != 0 {
                    0x80
                } else {
                    0x00
                };
                let op = self.get_operand(b);
                self.change_flag(Flag::Carry, op & 0x01 != 0);
                let result = op.rotate_right(1) as u8 | carry_in;
                self.set_zero_negative_flags(result);
                self.set_result(b, result);
            }

            Opcode::Sbc => {
                let operand = self.get_operand(b) as u8;
                let value = ((operand as i8).wrapping_neg().wrapping_sub(1)) as u8;
                self.add_to_a(value)
            }
            Opcode::Sta => {
                let addr = self.get_operand_address(b);
                debug!("STA a {:02x} into {:04x}", self.a, addr);
                self.bus.write_u8(addr, self.a);
            }

            Opcode::Stx => {
                let addr = self.get_operand_address(b);
                self.bus.write_u8(addr, self.x);
            }

            Opcode::Sty => {
                let addr = self.get_operand_address(b);
                self.bus.write_u8(addr, self.y);
            }

            Opcode::Lda => {
                self.a = self.get_operand(b) as u8;
                debug!("LDA a is {:02x}\n", self.a);
                self.set_zero_negative_flags(self.a);
            }

            Opcode::Ldx => {
                self.x = self.get_operand(b) as u8;
                debug!("LDX x is {:02x}", self.x);
                self.set_zero_negative_flags(self.x);
            }

            Opcode::Ldy => {
                self.y = self.get_operand(b) as u8;
                self.set_zero_negative_flags(self.y);
            }

            Opcode::Lsr => {
                let (result, carry_bit) = match b.instruction.mode {
                    Accumulator => {
                        let (result, carry_bit) = self.shift_left(self.a);
                        self.a = result;
                        (result, carry_bit)
                    }
                    _ => {
                        let addr = self.get_operand_address(b);
                        let value = self.bus.read_u8(addr);
                        let (result, carry_bit) = self.shift_left(value);
                        self.bus.write_u8(addr, result);
                        (result, carry_bit)
                    }
                };

                self.change_flag(Flag::Carry, carry_bit);
                self.set_zero_negative_flags(result);
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
            }
            Opcode::Tya => {
                self.a = self.y;
                self.set_zero_negative_flags(self.a);
            }

            Opcode::Jmp => {
                new_pc = self.get_operand_address(b);
                debug!("jmp to {:02X}", new_pc)
            }
            Opcode::Kil => {
                todo!("KIL")
            }
        }

        self.pc = new_pc;
    }

    fn get_operand(&mut self, b: &InstructionBytes) -> u16 {
        match b.instruction.mode {
            Immediate => b.get_immediate() as u16,
            Accumulator => self.a as u16,
            _ => {
                let addr = self.get_operand_address(b);
                let value = self.bus.read_u8(addr) as u16;
                debug!("get operand @ {:04x} = {:02x}", addr, value);
                value
            }
        }
    }

    /// get_operand returns either a value or address depending on mode
    /// references:
    /// https://www.nesdev.org/wiki/CPU_addressing_modes
    /// http://www.emulator101.com/6502-addressing-modes.html
    fn get_operand_address(&mut self, b: &InstructionBytes) -> u16 {
        match b.instruction.mode {
            ZeroPage => b.get_immediate() as u16,
            ZeroPageX => b.get_immediate().wrapping_add(self.x) as u16,
            ZeroPageY => b.get_immediate().wrapping_add(self.y) as u16,

            Absolute => b.get_address(),
            AbsoluteX => b.get_address().wrapping_add(self.x as u16),
            AbsoluteY => b.get_address().wrapping_add(self.y as u16),

            Indirect => {
                let addr = b.get_address();
                if addr & 0x00ff == 0x00ff {
                    let lsb = self.bus.read_u8(addr);
                    let msb = self.bus.read_u8(addr & 0xff00);
                    lsb as u16 | (msb as u16) << 8
                } else {
                    self.bus.read_u16(addr)
                }
            }
            IndirectX => {
                let addr = b.get_immediate().wrapping_add(self.x) as u8;
                let lsb = self.bus.read_u8(addr as u16);
                let msb = self.bus.read_u8(addr.wrapping_add(1) as u16);
                lsb as u16 | (msb as u16) << 8
            }
            IndirectY => {
                let addr = b.get_immediate();
                let lsb = self.bus.read_u8(addr as u16);
                let msb = self.bus.read_u8(addr.wrapping_add(1) as u16);
                let target = lsb as u16 | (msb as u16) << 8;
                target.wrapping_add(self.y as u16)
            }
            _ => panic!("get_operand not supported for {:?}", b.instruction.mode),
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
                mode: Relative,
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
            0x70 => Instruction {
                opcode: Opcode::Bvs,
                mode: Relative,
                length: 2,
                cycles: 2,
            },
            0x50 => Instruction {
                opcode: Opcode::Bvc,
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
                mode: Implied,
                length: 1,
                cycles: 2,
            },
            0xc8 => Instruction {
                opcode: Opcode::Iny,
                mode: Implied,
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
            0xa1 => Instruction {
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
            0x60 => Instruction {
                opcode: Opcode::Rts,
                mode: Implied,
                length: 1,
                cycles: 6,
            },
            0x40 => Instruction {
                opcode: Opcode::Rti,
                mode: Implied,
                length: 1,
                cycles: 6,
            },

            0x4a => Instruction {
                opcode: Opcode::Lsr,
                mode: Accumulator,
                length: 1,
                cycles: 2,
            },
            0x46 => Instruction {
                opcode: Opcode::Lsr,
                mode: ZeroPage,
                length: 2,
                cycles: 5,
            },
            0x56 => Instruction {
                opcode: Opcode::Lsr,
                mode: ZeroPageX,
                length: 2,
                cycles: 6,
            },
            0x4e => Instruction {
                opcode: Opcode::Lsr,
                mode: Absolute,
                length: 3,
                cycles: 6,
            },
            0x5e => Instruction {
                opcode: Opcode::Lsr,
                mode: AbsoluteX,
                length: 3,
                cycles: 7,
            },

            0x02 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x12 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x22 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x32 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x42 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x52 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x62 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x72 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x92 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0xB2 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0xD2 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0xF2 => Instruction {
                opcode: Opcode::Kil,
                mode: Implied,
                length: 1,
                cycles: 1,
            },
            0x1a => Instruction {
                opcode: Opcode::Nop,
                mode: Implied,
                length: 2,
                cycles: 1,
            },
            0x3a => Instruction {
                opcode: Opcode::Nop,
                mode: Implied,
                length: 2,
                cycles: 1,
            },
            0x5a => Instruction {
                opcode: Opcode::Nop,
                mode: Implied,
                length: 2,
                cycles: 1,
            },
            0x7a => Instruction {
                opcode: Opcode::Nop,
                mode: Implied,
                length: 2,
                cycles: 1,
            },
            0xda => Instruction {
                opcode: Opcode::Nop,
                mode: Implied,
                length: 2,
                cycles: 1,
            },
            0xfa => Instruction {
                opcode: Opcode::Nop,
                mode: Implied,
                length: 2,
                cycles: 1,
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
        // https://www.nesdev.org/wiki/CPU_power_up_state
        self.a = 0;
        self.x = 0;
        self.p = 0x24;
        self.pc = self.bus.read_u16(0xfffc);
        self.sp = 0xfd;
    }

    pub fn step(&mut self) -> u8 {
        let op = self.bus.read_u8(self.pc);
        let instruction = self.decode(op);
        let instruction_bytes = InstructionBytes {
            instruction: &instruction,
            bytes: self.bus.read_bytes(self.pc, instruction.length),
        };

        self.cycles = self.cycles.wrapping_add(instruction.cycles as u64);
        self.bus.tick(instruction.cycles);

        println!("{:04X}  {}   {}", self.pc, instruction_bytes, self);
        self.execute(&instruction_bytes);
        instruction.cycles
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.a, self.x, self.y, self.p, self.sp
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
    #[case(vec![0x09, 0x40], 0x84, 0xc4, 0b1010_0100)]
    #[case(vec![0x09, 0x00], 0x00, 0x00, 0b0010_0110)]
    #[case(vec![0x29, 0xf0], 0x80, 0x80, 0b1010_0100)]
    #[case(vec![0x49, 0xf0], 0x0f, 0xff, 0b1010_0100)]
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
    #[case(vec![0x10, 0x10], 0xc000, 0b1000_0000, 0b1000_0000, 0xc002)]
    #[case(vec![0x10, 0x10], 0xc000, 0b0000_0000, 0b0000_0000, 0xc012)]
    #[case(vec![0x10, 0xFB], 0xc000, 0b0000_0000, 0b0000_0000, 0xbffd)]
    #[case(vec![0xf0, 0x32], 0xc000, 0b1110_1111, 0b1110_1111, 0xc034)]

    fn test_branches(
        #[case] in_prg: Vec<u8>,
        #[case] in_pc: u16,
        #[case] in_flags: u8,
        #[case] ex_flags: u8,
        #[case] ex_pc: u16,
    ) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.p = in_flags;
        cpu.pc = in_pc;
        cpu.step();
        assert_eq!(cpu.p, ex_flags);
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

    #[test]
    fn test_stack() {
        let mut cpu = setup_cpu(test_program(vec![]));
        cpu.reset();
        cpu.stack_push_u8(0xda);
        assert_eq!(cpu.bus.read_u8(0x01fd), 0xda);
        let val = cpu.stack_pop_u8();
        assert_eq!(val, 0xda);
        cpu.stack_push_u16(0xda5c);
        assert_eq!(cpu.bus.read_u16(0x01fc), 0xda5c);
        let val = cpu.stack_pop_u16();
        assert_eq!(val, 0xda5c);
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
    #[case(vec![0xca], 0xff, 0, 0xfe, 0, 0b1010_0100)]
    #[case(vec![0x88], 0xa0, 0x05, 0xa0, 0x04, 0b0010_0100)]
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
    #[case(vec![0xe9, 0x41], 0x40, 0b11100101, 0xff, 0b10100100)]

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

    #[rstest]
    #[case(vec![0x2c, 0x03, 0xc0, 0xf0], 0xf0, 0b00000000, 0xf0, 0b11000000)]
    #[case(vec![0x2c, 0x03, 0xc0, 0x00], 0xf0, 0b00000000, 0xf0, 0b00000010)]
    fn test_bit(
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
    #[case(vec![0x4a], 0xff, 0b00000000, 0x7f, 0b00000001)]
    #[case(vec![0x4a], 0x80, 0b00000000, 0x40, 0b00000000)]
    fn test_lsr(
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
    #[case(vec![0x0a], 0x80, 0b00000000, 0x00, 0b00000011)]
    fn test_asl(
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
    #[case(vec![0xa5,0xff], vec![(0x00ff, 0x5a)], 0xff, 0xff, 0b00000000, 0x5a, 0b00000000)]
    #[case(vec![0xb5,0xef], vec![(0x00ff, 0x5a)], 0xff, 0x10, 0b00000000, 0x5a, 0b00000000)]
    #[case(vec![0xad,0x23,0x01], vec![(0x0123, 0x77)], 0xff, 0x10, 0b00000000, 0x77, 0b00000000)]
    #[case(vec![0xa1,0x10], vec![(0x0020, 0x23),(0x0021,0x01),(0x0123,0x5a)], 0xff, 0x10, 0b00000000, 0x5a, 0b00000000)]
    #[case(vec![0xa1,0xff], vec![(0x00ff, 0x23),(0x0000,0x01),(0x0123,0x5a)], 0xff, 0x00, 0b00000000, 0x5a, 0b00000000)]
    fn test_lda(
        #[case] in_prg: Vec<u8>,
        #[case] memory: Vec<(u16, u8)>,
        #[case] in_a: u8,
        #[case] in_x: u8,
        #[case] in_flags: u8,
        #[case] ex_a: u8,
        #[case] ex_flags: u8,
    ) {
        let mut cpu = setup_cpu(test_program(in_prg));
        cpu.reset();
        cpu.a = in_a;
        cpu.x = in_x;
        for (addr, mem) in memory {
            cpu.bus.write_u8(addr, mem);
        }
        cpu.p = in_flags;
        cpu.step();
        assert_eq!(cpu.a, ex_a);
        assert_eq!(cpu.p, ex_flags);
    }

    #[test]
    fn test_nmi() {
        let mut prog: Vec<u8> = vec![0x00];
        prog.append(&mut vec![0; 0x3ffa - 1]);
        prog.append(&mut vec![0x00, 0xaa]);
        prog.append(&mut vec![0x00, 0x80]);

        let mut cpu = setup_cpu(prog);
        cpu.reset();
        cpu.step();
        assert_eq!(cpu.pc, 0xaa00);
    }

    // TODO: write tests for RTS/JTS
    // TODO add more tests
}
