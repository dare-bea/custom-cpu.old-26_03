use std::{fmt::Display, num::Wrapping};

use crate::types::{Memory, RegByte, RegWord};

const GPR_COUNT: usize = 8;
const ADDR_MAX: usize = 0xFFFF;
const RAM_SIZE: usize = 0x8000;
const ROM_SIZE: usize = ADDR_MAX - RAM_SIZE + 1;
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
        self.gprs[reg as usize].0 = value
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
            };
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
            None
        } // ROM cannot be written to.
    }
}

impl Memory for [u8] {
    fn get(&self, address: u16) -> Option<u8> {
        <[u8]>::get(self, address as usize).copied()
    }
    fn set(&mut self, address: u16, value: u8) -> Option<()> {
        *<[u8]>::get_mut(self, address as usize)? = value;
        Some(())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmulatorError {
    MemoryLookupFail(u16),
    OpcodeUndefined(Vec<u8>),
    InvalidRegister(u8),
}

impl Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryLookupFail(addr) => write!(f, "failed to get data at address {addr:04x}"),
            Self::OpcodeUndefined(op) => write!(f, "opcode {op:02x?} undefined"),
            Self::InvalidRegister(id) => write!(f, "no register {id}"),
        }
    }
}

impl std::error::Error for EmulatorError {}

pub mod opcode {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum OperandLength {
        Byte = 1,
        Word = 2,
    }

    pub const fn is_nop(opcode: u8) -> bool {
        opcode == 0x00
    }
    pub const fn is_data(opcode: u8) -> bool {
        opcode & 0xFC != 0x04
    }
}

impl Emulator {
    pub fn step(&mut self) -> Result<(), EmulatorError> {
        use EmulatorError::*;
        let mut opcode = vec![self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?];
        if opcode::is_nop(opcode[0]) { /* NOP */
        } else if opcode::is_data(opcode[0]) {
            opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
            let (is_word, mode) = (
                opcode[0] & 0x02 == 0x02,
                (((opcode[0] & 0x01 == 0x01) as u8)
                    << (2 + ((opcode[1] & 0x80 == 0x80) as u8))
                    << (1 + ((opcode[1] & 0x40 == 0x40) as u8))),
            );
            let src = (opcode[1] >> 3) & 7;
            let dst = (opcode[1]) & 7;
            let src_regb = RegByte::try_from(src).map_err(|x| InvalidRegister(x.0));
            let src_regw = RegWord::try_from(src).map_err(|x| InvalidRegister(x.0));
            let dst_regb = RegByte::try_from(dst).map_err(|x| InvalidRegister(x.0));
            let dst_regw = RegWord::try_from(dst).map_err(|x| InvalidRegister(x.0));
            match mode {
                0 => {
                    /* LEA */
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    let value =
                        self.reg_word(src_regw?) + Wrapping(opcode[2].cast_signed() as i16 as u16);
                    if !is_word {
                        self.set_reg_byte(dst_regb?, value.0 as u8);
                    } else {
                        self.set_reg_word(dst_regw?, value.0);
                    }
                }
                1 => {
                    /* LDI */
                    if !is_word {
                        opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                        self.set_reg_byte(dst_regb?, opcode[2]);
                    } else {
                        opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                        opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                        self.set_reg_word(dst_regw?, u16::from_be_bytes([opcode[2], opcode[3]]));
                    }
                }
                2 => {
                    /* MOV */
                    if !is_word {
                        let value = self.reg_byte(src_regb?);
                        self.set_reg_byte(dst_regb?, value.0);
                    } else {
                        let value = self.reg_word(src_regw?);
                        self.set_reg_word(dst_regw?, value.0);
                    }
                }
                3 => {
                    /* LDI $0 */
                    if !is_word {
                        self.set_reg_byte(dst_regb?, 0);
                    } else {
                        self.set_reg_word(dst_regw?, 0);
                    }
                }
                4 => {
                    /* LD off */
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    let address = (self.reg_word(src_regw?)
                        + Wrapping(opcode[2].cast_signed() as i16 as u16))
                    .0;
                    if !is_word {
                        let value = self.get(address).ok_or(MemoryLookupFail(address))?;
                        self.set_reg_byte(dst_regb?, value);
                    } else {
                        let value = self.get_u16(address).ok_or(MemoryLookupFail(address))?;
                        self.set_reg_word(dst_regw?, value);
                    }
                }
                5 => {
                    /* LD addr */
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    let address = u16::from_be_bytes([opcode[2], opcode[3]]);
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    if !is_word {
                        let value = self.get(address).ok_or(MemoryLookupFail(address))?;
                        self.set_reg_byte(dst_regb?, value);
                    } else {
                        let value = self.get_u16(address).ok_or(MemoryLookupFail(address))?;
                        self.set_reg_word(dst_regw?, value);
                    }
                }
                6 => {
                    /* STO offs */
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    let address = (self.reg_word(src_regw?)
                        + Wrapping(opcode[2].cast_signed() as i16 as u16))
                    .0;
                    if !is_word {
                        let value = self.reg_byte(dst_regb?).0;
                        self.set(address, value).ok_or(MemoryLookupFail(address))?;
                    } else {
                        let value = self.reg_word(dst_regw?).0;
                        self.set_u16(address, value)
                            .ok_or(MemoryLookupFail(address))?;
                    }
                }
                7 => {
                    /* STO addr */
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    opcode.push(self.next_opcode().ok_or(MemoryLookupFail(self.pc.0))?);
                    let address = u16::from_be_bytes([opcode[2], opcode[3]]);
                    if !is_word {
                        let value = self.reg_byte(dst_regb?).0;
                        self.set(address, value).ok_or(MemoryLookupFail(address))?;
                    } else {
                        let value = self.reg_word(dst_regw?).0;
                        self.set_u16(address, value)
                            .ok_or(MemoryLookupFail(address))?;
                    }
                }
                _ => unreachable!(),
            }
        } else {
            return Err(OpcodeUndefined(opcode));
        }
        Ok(())
    }
}
