use std::{convert::Infallible, str::FromStr};
use thiserror::Error;

pub trait Bits {
    fn bits(&self) -> u32;
}

macro_rules! impl_bits_at_offset_inner {
    ($structname:ty, $offset:expr) => {
        impl Bits for $structname {
            fn bits(&self) -> u32 {
                self.0.bits() << $offset
            }
        }
    };
}

pub struct Or<A: Bits, B: Bits>(A, B);

impl<A: Bits, B: Bits> Bits for Or<A, B> {
    fn bits(&self) -> u32 {
        self.0.bits() | self.1.bits()
    }
}

impl<Rhs: Bits> core::ops::BitOr<Rhs> for Opcode {
    type Output = Or<Opcode, Rhs>;

    fn bitor(self, rhs: Rhs) -> Self::Output {
        Or(self, rhs)
    }
}

impl<A: Bits, B: Bits, Rhs: Bits> core::ops::BitOr<Rhs> for Or<A, B> {
    type Output = Or<Or<A, B>, Rhs>;

    fn bitor(self, rhs: Rhs) -> Self::Output {
        Or(self, rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Error)]
pub enum ParseImmidiateError {
    #[error("Failed to parse number")]
    InvalidNumber,

    #[error("Immidiate out of range")]
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Uimm<const BITS: usize>(pub u64);

impl<const BITS: usize> Uimm<BITS> {
    pub fn new(number: u64) -> Result<Self, ParseImmidiateError> {
        if BITS == 64 {
            return Ok(Self(number));
        }
        if number >= (1 << BITS) {
            return Err(ParseImmidiateError::OutOfRange);
        }

        Ok(Self(number))
    }
}

impl<const BITS: usize> FromStr for Uimm<BITS> {
    type Err = ParseImmidiateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number = if let Some(s) = s.strip_prefix("0x") {
            u64::from_str_radix(s, 16)
        } else {
            s.parse()
        }
        .map_err(|_| ParseImmidiateError::InvalidNumber)?;

        Self::new(number)
    }
}

impl<const BITS: usize> Bits for Uimm<BITS> {
    fn bits(&self) -> u32 {
        (self.0 & ((1 << BITS) - 1)) as u32
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Simm<const BITS: usize>(pub i64);

impl<const BITS: usize> Simm<BITS> {
    pub fn new(number: i64) -> Result<Self, ParseImmidiateError> {
        if number < -(1 << (BITS - 1)) {
            return Err(ParseImmidiateError::OutOfRange);
        }

        if number >= (1 << (BITS - 1)) {
            return Err(ParseImmidiateError::OutOfRange);
        }

        Ok(Self(number))
    }
}

impl<const BITS: usize> FromStr for Simm<BITS> {
    type Err = ParseImmidiateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, is_negative) = if let Some(s) = s.strip_prefix('-') {
            (s, true)
        } else {
            (s, false)
        };

        let number = if let Some(s) = s.strip_prefix("0x") {
            i64::from_str_radix(s, 16)
        } else {
            s.parse()
        }
        .map_err(|_| ParseImmidiateError::InvalidNumber)?;

        let number = if is_negative { -number } else { number };

        Self::new(number)
    }
}

impl<const BITS: usize> Bits for Simm<BITS> {
    fn bits(&self) -> u32 {
        self.0 as u32 & ((1 << BITS) - 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Opcode(pub Uimm<6>);

impl Opcode {
    pub fn new(number: u32) -> Result<Self, ParseImmidiateError> {
        Ok(Self(Uimm::new(number as u64)?))
    }

    pub fn fixed(number: u32) -> Self {
        Self(Uimm(number as u64))
    }
}

impl FromStr for Opcode {
    type Err = ParseImmidiateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl_bits_at_offset_inner!(Opcode, 26);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Funct(pub Uimm<11>);

impl Funct {
    pub fn new(number: u32) -> Result<Self, ParseImmidiateError> {
        Ok(Self(Uimm::new(number as u64)?))
    }

    pub fn fixed(number: u32) -> Self {
        Self(Uimm(number as u64))
    }
}

impl FromStr for Funct {
    type Err = ParseImmidiateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl_bits_at_offset_inner!(Funct, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Off9(pub Uimm<9>);

impl FromStr for Off9 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uimm11: Uimm<11> = s.parse()?;
        anyhow::ensure!(uimm11.0 & 0x3 == 0, "unaligned offset");
        Ok(Self(Uimm(uimm11.0 >> 2)))
    }
}

impl_bits_at_offset_inner!(Off9, 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Off14(pub Uimm<14>);

impl FromStr for Off14 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uimm16: Uimm<16> = s.parse()?;
        anyhow::ensure!(uimm16.0 & 0x3 == 0, "unaligned offset");
        Ok(Self(Uimm(uimm16.0 >> 2)))
    }
}

impl_bits_at_offset_inner!(Off14, 2);

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum ParseRegisterError {
    #[error("Invalid Register")]
    InvalidRegister,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Reg(pub u32);

impl FromStr for Reg {
    type Err = ParseRegisterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "zero" {
            return Ok(Reg(0));
        }

        let (start, rest) = s.split_at(1);
        if start != "r" {
            return Err(ParseRegisterError::InvalidRegister);
        }

        let num: u32 = rest
            .parse()
            .map_err(|_| ParseRegisterError::InvalidRegister)?;
        if num > 31 {
            return Err(ParseRegisterError::InvalidRegister);
        }

        Ok(Self(num))
    }
}

impl Bits for Reg {
    fn bits(&self) -> u32 {
        self.0
    }
}

macro_rules! impl_register {
    ($structname:ty, $offset:expr) => {
        impl FromStr for $structname {
            type Err = ParseRegisterError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse::<Reg>()?))
            }
        }

        impl_bits_at_offset_inner!($structname, $offset);
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Rd(pub Reg);
impl_register!(Rs, 21);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Rs(pub Reg);
impl_register!(Rd, 16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Rt(pub Reg);
impl_register!(Rt, 11);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Jmpop {
    Jump,
    Call,
}

impl Bits for Jmpop {
    fn bits(&self) -> u32 {
        let bits = match *self {
            Jmpop::Call => 0x0,
            Jmpop::Jump => 0x1,
        };

        bits << 24
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Label(pub String);

impl FromStr for Label {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Label(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uimm() {
        assert_eq!("0x00".parse::<Uimm<8>>(), Ok(Uimm(0)));
        assert_eq!("10".parse::<Uimm<8>>(), Ok(Uimm(10)));
        assert_eq!("255".parse::<Uimm<8>>(), Ok(Uimm(255)));
        assert_eq!(
            "256".parse::<Uimm<8>>(),
            Err(ParseImmidiateError::OutOfRange)
        );
        assert_eq!(
            "-10".parse::<Uimm<8>>(),
            Err(ParseImmidiateError::InvalidNumber)
        );
        assert_eq!("0x1ffff".parse::<Uimm<17>>(), Ok(Uimm(0x1ffff)));
    }

    #[test]
    fn bits_uimm() {
        assert_eq!(Uimm::<11>(10).bits(), 10u32);
        assert_eq!(Uimm::<11>(0x7ff).bits(), 0x7ffu32);
        assert_eq!(Uimm::<11>(0xfff).bits(), 0x7ffu32);
    }

    #[test]
    fn parse_simm() {
        assert_eq!("0x00".parse::<Simm<8>>(), Ok(Simm(0)));
        assert_eq!("10".parse::<Simm<8>>(), Ok(Simm(10)));
        assert_eq!(
            "255".parse::<Simm<8>>(),
            Err(ParseImmidiateError::OutOfRange)
        );
        assert_eq!(
            "256".parse::<Simm<8>>(),
            Err(ParseImmidiateError::OutOfRange)
        );
        assert_eq!("-10".parse::<Simm<8>>(), Ok(Simm(-10)));
        assert_eq!(
            "0x1ffff".parse::<Simm<17>>(),
            Err(ParseImmidiateError::OutOfRange)
        );
        assert_eq!("0xffff".parse::<Simm<17>>(), Ok(Simm(0xffff)));
        assert_eq!(
            "0x10000".parse::<Simm<17>>(),
            Err(ParseImmidiateError::OutOfRange)
        );
        assert_eq!("-0x10000".parse::<Simm<17>>(), Ok(Simm(-0x10000)));
        assert_eq!(
            "-0x10001".parse::<Simm<17>>(),
            Err(ParseImmidiateError::OutOfRange)
        );
    }

    #[test]
    fn bits_simm() {
        assert_eq!(Simm::<11>(10).bits(), 10u32);
        assert_eq!(Simm::<11>(0x7ff).bits(), 0x7ffu32);
        assert_eq!(Simm::<11>(-10).bits(), 0x000007f6);
    }

    #[test]
    fn parse_opcode() {
        assert_eq!("0x00".parse::<Opcode>(), Ok(Opcode(Uimm(0))));
        assert_eq!("0x3f".parse::<Opcode>(), Ok(Opcode(Uimm(63))));
        assert_eq!("32".parse::<Opcode>(), Ok(Opcode(Uimm(32))));
        assert_eq!("64".parse::<Opcode>(), Err(ParseImmidiateError::OutOfRange));
        assert_eq!(
            "-1".parse::<Opcode>(),
            Err(ParseImmidiateError::InvalidNumber)
        );
    }

    #[test]
    fn bits_opcode() {
        assert_eq!(Opcode(Uimm(0x00)).bits(), 0x00000000u32);
        assert_eq!(Opcode(Uimm(0x01)).bits(), 0x04000000u32);
        assert_eq!(Opcode(Uimm(0x02)).bits(), 0x08000000u32);
        assert_eq!(Opcode(Uimm(0x04)).bits(), 0x10000000u32);
        assert_eq!(Opcode(Uimm(0x08)).bits(), 0x20000000u32);
        assert_eq!(Opcode(Uimm(0x10)).bits(), 0x40000000u32);
        assert_eq!(Opcode(Uimm(0x20)).bits(), 0x80000000u32);
        assert_eq!(Opcode(Uimm(0x3e)).bits(), 0xf8000000u32);
    }

    #[test]
    fn parse_reg() {
        assert_eq!("zero".parse::<Reg>(), Ok(Reg(0)));
    }

    #[test]
    fn bits_rd() {
        assert_eq!(Rd(Reg(1)).bits(), 0x00010000u32);
        assert_eq!(Rd(Reg(2)).bits(), 0x00020000u32);
        assert_eq!(Rd(Reg(4)).bits(), 0x00040000u32);
        assert_eq!(Rd(Reg(8)).bits(), 0x00080000u32);
        assert_eq!(Rd(Reg(16)).bits(), 0x00100000u32);
        assert_eq!(Rd(Reg(27)).bits(), 0x001b0000u32);
    }

    #[test]
    fn bits_rs() {
        assert_eq!(Rs(Reg(1)).bits(), 0x00200000u32);
        assert_eq!(Rs(Reg(2)).bits(), 0x00400000u32);
        assert_eq!(Rs(Reg(4)).bits(), 0x00800000u32);
        assert_eq!(Rs(Reg(8)).bits(), 0x01000000u32);
        assert_eq!(Rs(Reg(16)).bits(), 0x02000000u32);
        assert_eq!(Rs(Reg(27)).bits(), 0x03600000u32);
    }

    #[test]
    fn bits_rt() {
        assert_eq!(Rt(Reg(1)).bits(), 0x00000800u32);
        assert_eq!(Rt(Reg(2)).bits(), 0x00001000u32);
        assert_eq!(Rt(Reg(4)).bits(), 0x00002000u32);
        assert_eq!(Rt(Reg(8)).bits(), 0x00004000u32);
        assert_eq!(Rt(Reg(16)).bits(), 0x00008000u32);
        assert_eq!(Rt(Reg(27)).bits(), 0x0000d800u32);
    }

    #[test]
    fn bits_jmpop() {
        assert_eq!(Jmpop::Call.bits(), 0x00000000);
        assert_eq!(Jmpop::Jump.bits(), 0x01000000);
    }
}
