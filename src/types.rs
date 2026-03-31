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

pub struct InvalidRegister(pub u8);

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
    pub fn as_mask(&self) -> u16 {
        1 << (*self as u8)
    }
}

pub trait Memory {
    #[must_use]
    fn get(&self, address: u16) -> Option<u8>;
    #[must_use]
    fn set(&mut self, address: u16, value: u8) -> Option<()>;

    #[must_use]
    fn get_u16(&self, address: u16) -> Option<u16> {
        Some(u16::from_be_bytes([
            self.get(address)?,
            self.get(address + 1)?,
        ]))
    }
    #[must_use]
    fn set_u16(&mut self, address: u16, value: u16) -> Option<()> {
        let [high, low] = value.to_be_bytes();
        self.set(address, high)?;
        self.set(address + 1, low)
    }
}
