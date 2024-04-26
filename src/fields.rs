use std::str::FromStr;
use thiserror::Error;

pub trait Bits {
    fn bits(&self) -> u32;
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

impl<A: Bits, B:Bits, Rhs: Bits> core::ops::BitOr<Rhs> for Or<A, B> {
    type Output = Or<Or<A, B>, Rhs>;

    fn bitor(self, rhs: Rhs) -> Self::Output {
        Or(self, rhs)
    }
}


#[derive(Debug, Error)]
pub enum ParseOpcodeError {
    #[error("Failed to parse opcode number")]
    ParseOpcodeError
}

pub struct Opcode(pub u32);

impl FromStr for Opcode {
    type Err = ParseOpcodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number = if let Some(s) = s.strip_prefix("0x") {
            u32::from_str_radix(s, 16)
        } else {
            u32::from_str_radix(s, 10)
        }.map_err(|_| ParseOpcodeError::ParseOpcodeError)?;

        Ok(Opcode(number))
    }
}

impl Bits for Opcode {
    fn bits(&self) -> u32 {
        self.0 << 26
    }
}

#[derive(Debug, Error)]
pub enum ParseImmidiateError {
    #[error("Failed to parse number")]
    InvalidNumber,

    #[error("Immidiate out of range")]
    OutOfRange
}

pub struct Uimm<const BITS: usize>(u32);
pub struct Simm<const BITS: usize>(i32);

impl<const BITS: usize> FromStr for Uimm<BITS> {
    type Err = ParseImmidiateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number = if let Some(s) = s.strip_prefix("0x") {
            u32::from_str_radix(s, 16)
        } else {
            u32::from_str_radix(s, 10)
        }.map_err(|_| ParseImmidiateError::InvalidNumber)?;

        if number >= (1 << BITS) {
            return Err(ParseImmidiateError::OutOfRange);
        }

        Ok(Self(number))
    }
}

impl<const BITS: usize> Bits for Uimm<BITS> {
    fn bits(&self) -> u32 {
        self.0
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
            i32::from_str_radix(s, 16)
        } else {
            i32::from_str_radix(s, 10)
        }.map_err(|_| ParseImmidiateError::InvalidNumber)?;

        let number = if is_negative { -number } else { number };

        if number < -(1 << (BITS - 1)) {
            return Err(ParseImmidiateError::OutOfRange);
        }

        if number >= (1 << (BITS - 1)) {
            return Err(ParseImmidiateError::OutOfRange);
        }

        Ok(Self(number & ((1 << BITS) - 1)))
    }
}

impl<const BITS: usize> Bits for Simm<BITS> {
    fn bits(&self) -> u32 {
        self.0 as u32
    }
}

#[derive(Debug, Error)]
pub enum ParseRegisterError {
    #[error("Invalid Register")]
    InvalidRegister
}

pub struct Reg(u32);

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

        let num: u32 = rest.parse().map_err(|_| ParseRegisterError::InvalidRegister)?;
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

pub struct Rd(Reg);
pub struct Rs(Reg);
pub struct Rt(Reg);

macro_rules! impl_register_bits {
    ($structname:ty, $offset:expr) => {
        impl FromStr for $structname {
            type Err = ParseRegisterError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse::<Reg>()?))
            }
        }

        impl Bits for $structname {
            fn bits(&self) -> u32 {
                self.0.bits() << $offset
            }
        }
    }
}

impl_register_bits!(Rs, 21);
impl_register_bits!(Rd, 16);
impl_register_bits!(Rt, 11);