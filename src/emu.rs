use std::{fmt::Display, num::Wrapping};

use crate::types::{Condition, Flags, Memory, RegByte, RegWord};

const GPR_COUNT: usize = 8;
const RAM_SIZE: usize = 0x8000;
const ROM_SIZE: usize = 0x10000 - RAM_SIZE;
const PC_START: u16 = 0x8000;
const SP_START: u16 = 0x8000;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Emulator {
    gprs: [Wrapping<u8>; GPR_COUNT],
    sp: Wrapping<u16>,
    pc: Wrapping<u16>,
    flags: Wrapping<u16>,
    ram: [u8; RAM_SIZE],
    rom: [u8; ROM_SIZE],
}

impl Emulator {
    pub fn new(rom: [u8; ROM_SIZE]) -> Self {
        Emulator {
            gprs: [Wrapping(0); GPR_COUNT],
            sp: Wrapping(SP_START),
            pc: Wrapping(PC_START),
            flags: Wrapping(0),
            ram: [0; RAM_SIZE],
            rom,
        }
    }

    pub fn next_opcode(&mut self) -> Option<u8> {
        let opcode = self.get(self.pc.0);
        self.pc += 1;
        opcode
    }

    #[inline]
    pub fn reg_byte(&self, reg: RegByte) -> Wrapping<u8> {
        self.gprs[reg as usize]
    }

    #[inline]
    pub fn set_reg_byte(&mut self, reg: RegByte, value: u8) {
        self.gprs[reg as usize].0 = value;
    }

    #[inline]
    pub fn reg_word(&self, reg: RegWord) -> Wrapping<u16> {
        if let Some((high, low)) = reg.gpr_parts() {
            Wrapping(u16::from_be_bytes([
                self.gprs[high as usize].0,
                self.gprs[low as usize].0,
            ]))
        } else {
            match reg {
                RegWord::PC => self.pc,
                RegWord::SP => self.sp,
                RegWord::FLAGS => self.flags,
                _ => unreachable!(),
            }
        }
    }

    #[inline]
    pub fn set_reg_word(&mut self, reg: RegWord, value: u16) {
        if let Some((high, low)) = reg.gpr_parts() {
            [self.gprs[high as usize].0, self.gprs[low as usize].0] = value.to_be_bytes();
        } else {
            match reg {
                RegWord::PC => self.pc.0 = value,
                RegWord::SP => self.sp.0 = value,
                RegWord::FLAGS => self.flags.0 = value,
                _ => unreachable!(),
            }
        }
    }
}

impl Memory for Emulator {
    fn get(&self, address: u16) -> Option<u8> {
        if address < RAM_SIZE as u16 {
            Memory::get(self.ram.as_slice(), address)
        } else {
            Memory::get(self.rom.as_slice(), address - RAM_SIZE as u16)
        }
    }

    fn set(&mut self, address: u16, value: u8) -> Option<()> {
        if address < RAM_SIZE as u16 {
            Memory::set(self.ram.as_mut_slice(), address, value)
        } else {
            None // ROM cannot be written to.
        }
    }
}

impl Memory for [u8] {
    fn get(&self, address: u16) -> Option<u8> {
        self.get(address as usize).copied()
    }

    fn set(&mut self, address: u16, value: u8) -> Option<()> {
        *self.get_mut(address as usize)? = value;
        Some(())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmulatorError {
    MemoryLookupFail(u16),
    OpcodeUndefined(Vec<u8>),
    InvalidRegister(u8),
    InvalidCondition(u8),
}

impl Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryLookupFail(addr) => write!(f, "failed to get data at address {addr:04x}"),
            Self::OpcodeUndefined(op) => write!(f, "opcode {op:02x?} undefined"),
            Self::InvalidRegister(id) => write!(f, "no register {id}"),
            Self::InvalidCondition(id) => write!(f, "no condition {id}"),
        }
    }
}

impl std::error::Error for EmulatorError {}

enum AluError {
    EmulatorError(EmulatorError),
    MissingOperation,
}

impl AluError {
    pub fn to_emulator_error(self, opcodes: Vec<u8>) -> EmulatorError {
        match self {
            Self::EmulatorError(e) => e,
            Self::MissingOperation => EmulatorError::OpcodeUndefined(opcodes),
        }
    }
}

impl From<EmulatorError> for AluError {
    fn from(value: EmulatorError) -> Self {
        Self::EmulatorError(value)
    }
}

pub mod opcode {
    /// An opcode of `0xFF` is a NOP.
    pub const fn is_nop(opcode: u8) -> bool {
        opcode == 0xFF
    }

    /// A data opcode is of the form 0b0001_01xx.
    pub const fn is_data(opcode: u8) -> bool {
        opcode & 0xFC == 0x14
    }

    /// A jump opcode is of the form 0b0001_00xx.
    pub const fn is_jump(opcode: u8) -> bool {
        opcode & 0xFC == 0x10
    }

    /// A binary ALU opcode is of the form 0b0000_1xxx.
    pub const fn is_alu_binary(opcode: u8) -> bool {
        opcode & 0xF8 == 0x08
    }

    /// A unary ALU opcode is of the form 0b0000_0xxx.
    pub const fn is_alu_unary(opcode: u8) -> bool {
        opcode & 0xF8 == 0x00
    }
}

impl Emulator {
    fn push_next_opcode(&mut self, opcodes: &mut Vec<u8>) -> Result<(), EmulatorError> {
        opcodes.push(
            self.next_opcode()
                .ok_or(EmulatorError::MemoryLookupFail(self.pc.0))?,
        );
        Ok(())
    }
    
        pub const fn check_flag(&self, flag: Flags) -> bool {
        self.flags.0 & flag.as_mask() != 0
    }

    pub const fn test_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Z => self.check_flag(Flags::ZF),
            Condition::C => self.check_flag(Flags::CF),
            Condition::S => self.check_flag(Flags::SF),
            Condition::O => self.check_flag(Flags::OF),
            Condition::BE => self.check_flag(Flags::ZF) || self.check_flag(Flags::CF),
            Condition::L => self.check_flag(Flags::SF) ^ self.check_flag(Flags::OF),
            Condition::LE => {
                (self.check_flag(Flags::SF) ^ self.check_flag(Flags::OF))
                    || self.check_flag(Flags::ZF)
            }
            Condition::NZ => !self.check_flag(Flags::ZF),
            Condition::NC => !self.check_flag(Flags::CF),
            Condition::NS => !self.check_flag(Flags::SF),
            Condition::NO => !self.check_flag(Flags::OF),
            Condition::NBE => !self.check_flag(Flags::ZF) && !self.check_flag(Flags::CF),
            Condition::NL => !(self.check_flag(Flags::SF) ^ self.check_flag(Flags::OF)),
            Condition::NLE => {
                !(self.check_flag(Flags::SF) ^ self.check_flag(Flags::OF))
                    && !self.check_flag(Flags::ZF)
            }
        }
    }

    pub fn step(&mut self) -> Result<(), EmulatorError> {
        use EmulatorError::*;

        let opcodes = vec![self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?];

        if opcode::is_nop(opcodes[0]) {
            // NOP - nothing to do.
            Ok(())
        } else if opcode::is_data(opcodes[0]) {
            // One of LDI, MOV, LD, or STO.
            self.step_data(opcodes)
        } else if opcode::is_jump(opcodes[0]) {
            // JMP or JCC.
            self.step_jump(opcodes)
        } else if opcode::is_alu_binary(opcodes[0]) {
            // Binary ALU operation.
            self.step_alu_binary(opcodes)
        } else if opcode::is_alu_unary(opcodes[0]) {
            // Unary ALU operation.
            self.step_alu_unary(opcodes)
        } else {
            Err(OpcodeUndefined(opcodes))
        }
    }

    fn step_data(&mut self, mut opcodes: Vec<u8>) -> Result<(), EmulatorError> {
        use EmulatorError::*;

        self.push_next_opcode(&mut opcodes)?;

        let is_word = opcodes[0] & 0x02 != 0;
        // mode is a 3-bit value: { opcode[0][0], opcode[1][7], opcode[1][6] }
        let mode = ((opcodes[0] & 0x01) << 2) | (opcodes[1] >> 6);

        let src = (opcodes[1] >> 3) & 0x07;
        let dst = opcodes[1] & 0x07;

        let src_regb = || RegByte::try_from(src).map_err(|x| InvalidRegister(x.0));
        let src_regw = || RegWord::try_from(src).map_err(|x| InvalidRegister(x.0));
        let dst_regb = || RegByte::try_from(dst).map_err(|x| InvalidRegister(x.0));
        let dst_regw = || RegWord::try_from(dst).map_err(|x| InvalidRegister(x.0));

        match mode {
            0 => {
                // LEA off(%src), %dst:
                // dst := src + off
                self.push_next_opcode(&mut opcodes)?;
                let offset = opcodes[2].cast_signed() as i16 as u16;
                let value = self.reg_word(src_regw()?) + Wrapping(offset);
                if !is_word {
                    self.set_reg_byte(dst_regb()?, value.0 as u8);
                } else {
                    self.set_reg_word(dst_regw()?, value.0);
                }
            }
            1 => {
                // LDI $imm, %dst:
                // dst := imm
                if !is_word {
                    self.push_next_opcode(&mut opcodes)?;
                    self.set_reg_byte(dst_regb()?, opcodes[2]);
                } else {
                    self.push_next_opcode(&mut opcodes)?;
                    self.push_next_opcode(&mut opcodes)?;
                    self.set_reg_word(dst_regw()?, u16::from_be_bytes([opcodes[2], opcodes[3]]));
                }
            }
            2 => {
                // MOV %src, %dst:
                // dst := src
                if !is_word {
                    let value = self.reg_byte(src_regb()?);
                    self.set_reg_byte(dst_regb()?, value.0);
                } else {
                    let value = self.reg_word(src_regw()?);
                    self.set_reg_word(dst_regw()?, value.0);
                }
            }
            3 => {
                // LDI $0, %dst:
                // dst := 0
                if !is_word {
                    self.set_reg_byte(dst_regb()?, 0);
                } else {
                    self.set_reg_word(dst_regw()?, 0);
                }
            }
            4 => {
                // LD off(%src), %dst:
                // dst := *(src + off)
                self.push_next_opcode(&mut opcodes)?;
                let offset = opcodes[2].cast_signed() as i16 as u16;
                let address = (self.reg_word(src_regw()?) + Wrapping(offset)).0;
                if !is_word {
                    let value = self.get(address).ok_or(MemoryLookupFail(address))?;
                    self.set_reg_byte(dst_regb()?, value);
                } else {
                    let value = self.get_word(address).ok_or(MemoryLookupFail(address))?;
                    self.set_reg_word(dst_regw()?, value);
                }
            }
            5 => {
                // LD addr, %dst:
                // dst := *addr
                self.push_next_opcode(&mut opcodes)?;
                self.push_next_opcode(&mut opcodes)?;
                let address = u16::from_be_bytes([opcodes[2], opcodes[3]]);
                if !is_word {
                    let value = self.get(address).ok_or(MemoryLookupFail(address))?;
                    self.set_reg_byte(dst_regb()?, value);
                } else {
                    let value = self.get_word(address).ok_or(MemoryLookupFail(address))?;
                    self.set_reg_word(dst_regw()?, value);
                }
            }
            6 => {
                // STO %src, off(%dst): %dst is at %src bitmask
                // *(dst + off) := src
                self.push_next_opcode(&mut opcodes)?;
                let offset = opcodes[2].cast_signed() as i16 as u16;
                let address = (self.reg_word(src_regw()?) + Wrapping(offset)).0;
                if !is_word {
                    let value = self.reg_byte(dst_regb()?).0;
                    self.set(address, value).ok_or(MemoryLookupFail(address))?;
                } else {
                    let value = self.reg_word(dst_regw()?).0;
                    self.set_word(address, value)
                        .ok_or(MemoryLookupFail(address))?;
                }
            }
            7 => {
                // STO %src, addr: %dst is at %src bitmask
                // *addr := src
                self.push_next_opcode(&mut opcodes)?;
                self.push_next_opcode(&mut opcodes)?;
                let address = u16::from_be_bytes([opcodes[2], opcodes[3]]);
                if !is_word {
                    let value = self.reg_byte(dst_regb()?).0;
                    self.set(address, value).ok_or(MemoryLookupFail(address))?;
                } else {
                    let value = self.reg_word(dst_regw()?).0;
                    self.set_word(address, value)
                        .ok_or(MemoryLookupFail(address))?;
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn step_jump(&mut self, mut opcodes: Vec<u8>) -> Result<(), EmulatorError> {
        use EmulatorError::*;

        self.push_next_opcode(&mut opcodes)?;

        let condition = opcodes[1] >> 4;
        let is_offs: bool = (opcodes[1] >> 3) & 1 != 0;
        let src = opcodes[1] & 7;

        let src_regw = || RegWord::try_from(src).map_err(|x| InvalidRegister(x.0));

        let address = if is_offs {
            self.push_next_opcode(&mut opcodes)?;
            (self.reg_word(src_regw()?) + Wrapping(opcodes[2].cast_signed() as i16 as u16)).0
        } else {
            self.push_next_opcode(&mut opcodes)?;
            self.push_next_opcode(&mut opcodes)?;
            u16::from_be_bytes([opcodes[2], opcodes[3]])
        };

        if (opcodes[0] & 1 == 0)
            || self
                .test_condition(Condition::try_from(condition).map_err(|x| InvalidCondition(x.0))?)
        {
            self.pc = Wrapping(address);
        }

        Ok(())
    }

    fn step_alu_binary(&mut self, mut opcodes: Vec<u8>) -> Result<(), EmulatorError> {
        use EmulatorError::*;

        self.push_next_opcode(&mut opcodes)?;

        let operation = opcodes[0] & 7;
        let is_word = opcodes[1] & 0x80 != 0;
        let is_imm = opcodes[1] & 0x40 != 0;

        let src = (opcodes[1] >> 3) & 0x07;
        let dst = opcodes[1] & 0x07;

        let src_regb = || RegByte::try_from(src).map_err(|x| InvalidRegister(x.0));
        let src_regw = || RegWord::try_from(src).map_err(|x| InvalidRegister(x.0));
        let dst_regb = || RegByte::try_from(dst).map_err(|x| InvalidRegister(x.0));
        let dst_regw = || RegWord::try_from(dst).map_err(|x| InvalidRegister(x.0));

        if is_word {
            let lhs = self.reg_byte(dst_regb()?).0;
            let rhs = if is_imm {
                self.push_next_opcode(&mut opcodes)?;
                opcodes[2]
            } else {
                self.reg_byte(src_regb()?).0
            };
            let value = self.step_alu_binary_byte(operation, lhs, rhs).map_err(|e| e.to_emulator_error(opcodes))?;
            self.set_reg_byte(dst_regb()?, value);
        } else {
            let lhs = self.reg_word(dst_regw()?).0;
            let rhs = if is_imm {
                self.push_next_opcode(&mut opcodes)?;
                self.push_next_opcode(&mut opcodes)?;
                u16::from_be_bytes([opcodes[2], opcodes[3]])
            } else {
                self.reg_word(src_regw()?).0
            };
            let value = self.step_alu_binary_word(operation, lhs, rhs).map_err(|e| e.to_emulator_error(opcodes))?;
            self.set_reg_word(dst_regw()?, value);
        }

        Ok(())
    }

    fn step_alu_binary_byte(&mut self, operation: u8, lhs: u8, rhs: u8) -> Result<u8, AluError> {
        Ok(match operation {
            0 => {
                // add
                self.flags.0 &= !(
                    Flags::ZF.as_mask()
                    | Flags::CF.as_mask()
                    | Flags::SF.as_mask()
                    | Flags::OF.as_mask()
                );
                let oflag = lhs.cast_signed().checked_add(rhs.cast_signed()).is_none();
                let (value, cflag) = lhs.overflowing_add(rhs);
                if value == 0 {self.flags.0 |= Flags::ZF.as_mask();}
                if value.cast_signed().is_negative() {self.flags.0 |= Flags::SF.as_mask();}
                if oflag {self.flags.0 |= Flags::OF.as_mask();}
                if cflag {self.flags.0 |= Flags::CF.as_mask();}
                value
            }
            1 => {
                // sub
                self.flags.0 &= !(
                    Flags::ZF.as_mask()
                    | Flags::CF.as_mask()
                    | Flags::SF.as_mask()
                    | Flags::OF.as_mask()
                );
                let oflag = lhs.cast_signed().checked_sub(rhs.cast_signed()).is_none();
                let (value, cflag) = lhs.overflowing_sub(rhs);
                if value == 0 {self.flags.0 |= Flags::ZF.as_mask();}
                if value.cast_signed().is_negative() {self.flags.0 |= Flags::SF.as_mask();}
                if oflag {self.flags.0 |= Flags::OF.as_mask();}
                if cflag {self.flags.0 |= Flags::CF.as_mask();}
                value
            }
            _ => return Err(AluError::MissingOperation)
        })
    }

    fn step_alu_binary_word(&mut self, _operation: u8, _lhs: u16, _rhs: u16) -> Result<u16, AluError> {
        todo!()
    }

    fn step_alu_unary(&mut self, _opcodes: Vec<u8>) -> Result<(), EmulatorError> {
        todo!()
    }
}
