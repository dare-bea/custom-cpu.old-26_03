use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum RegByte {
    H = 0,
    A = 1,
    B = 2,
    C = 3,
    X = 4,
    L = 5,
    M = 6,
    N = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidRegister(pub u8);
impl fmt::Display for InvalidRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid register {}", self.0)
    }
}
impl std::error::Error for InvalidRegister {}

impl TryFrom<u8> for RegByte {
    type Error = InvalidRegister;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::H,
            1 => Self::A,
            2 => Self::B,
            3 => Self::C,
            4 => Self::X,
            5 => Self::L,
            6 => Self::M,
            7 => Self::N,
            _ => return Err(InvalidRegister(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum RegWord {
    HA = 0,
    BC = 1,
    XL = 2,
    MN = 3,
    SP = 5,
    PC = 6,
    FLAGS = 7,
}

impl RegWord {
    pub const fn gpr_parts(&self) -> Option<(RegByte, RegByte)> {
        use RegByte::*;
        Some(match self {
            Self::HA => (H, A),
            Self::BC => (B, C),
            Self::XL => (X, L),
            Self::MN => (M, N),
            _ => return None,
        })
    }
}

impl TryFrom<u8> for RegWord {
    type Error = InvalidRegister;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::HA,
            1 => Self::BC,
            2 => Self::XL,
            3 => Self::MN,
            5 => Self::SP,
            6 => Self::PC,
            7 => Self::FLAGS,
            _ => return Err(InvalidRegister(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Flags {
    ZF = 0,
    CF = 1,
    SF = 2,
    OF = 3,
    HF = 4,
}

impl Flags {
    pub const fn as_mask(&self) -> u16 {
        1 << (*self as u8)
    }
}

pub trait Memory {
    #[must_use]
    fn get(&self, address: u16) -> Option<u8>;
    #[must_use]
    fn set(&mut self, address: u16, value: u8) -> Option<()>;

    #[must_use]
    fn get_word(&self, address: u16) -> Option<u16> {
        Some(u16::from_be_bytes([
            self.get(address)?,
            self.get(address + 1)?,
        ]))
    }
    #[must_use]
    fn set_word(&mut self, address: u16, value: u16) -> Option<()> {
        let [high, low] = value.to_be_bytes();
        self.set(address, high)?;
        self.set(address + 1, low)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidCondition(pub u8);
impl fmt::Display for InvalidCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid condition {}", self.0)
    }
}
impl std::error::Error for InvalidCondition {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Condition {
    Z = 0o00,
    NZ = 0o10,
    C = 0o01,
    NC = 0o11,
    S = 0o02,
    NS = 0o12,
    O = 0o03,
    NO = 0o13,
    BE = 0o05,
    NBE = 0o15,
    L = 0o06,
    NL = 0o16,
    LE = 0o07,
    NLE = 0o17,
}

impl TryFrom<u8> for Condition {
    type Error = InvalidCondition;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0o00 => Self::Z,
            0o10 => Self::NZ,
            0o01 => Self::C,
            0o11 => Self::NC,
            0o02 => Self::S,
            0o12 => Self::NS,
            0o03 => Self::O,
            0o13 => Self::NO,
            0o05 => Self::BE,
            0o15 => Self::NBE,
            0o06 => Self::L,
            0o16 => Self::NL,
            0o07 => Self::LE,
            0o17 => Self::NLE,
            _ => Err(InvalidCondition(value))?,
        })
    }
}

impl std::ops::Not for Condition {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Z => Self::NZ,
            Self::NZ => Self::Z,
            Self::C => Self::NC,
            Self::NC => Self::C,
            Self::S => Self::NS,
            Self::NS => Self::S,
            Self::O => Self::NO,
            Self::NO => Self::O,
            Self::BE => Self::NBE,
            Self::NBE => Self::BE,
            Self::L => Self::NL,
            Self::NL => Self::L,
            Self::LE => Self::NLE,
            Self::NLE => Self::LE,
        }
    }
}
