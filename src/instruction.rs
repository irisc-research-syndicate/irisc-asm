use core::str::FromStr;

use anyhow::{bail, ensure};

use crate::fields::{Bits, Opcode, Rd, Rs, Rt, Uimm, Simm};

pub enum Instruction {
    Label(String),
    Unki(Opcode, Rd, Rs, Simm<16>),
    Unkr(Opcode, Rd, Rs, Rt, Simm<11>),
    Addi(Rd, Rs, Simm<16>),
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let line = line.trim();
        let Some((cmd, rest)) = line.split_once(' ') else {
            bail!("Bad instruction: {}", line)
        };

        let params = rest.trim().split(',').map(|p| p.trim()).collect::<Vec<_>>();

        Ok(match cmd {
            "lbl" => {
                ensure!(params.len() == 1, "Wrong number of parameters");
                Instruction::Label(params[0].to_string())
            },
            "unk.i" => {
                ensure!(params.len() == 4, "Wrong number of parameters");
                Instruction::Unki(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?)
            }
            "unk.r" => {
                ensure!(params.len() == 5, "Wrong number of parameters");
                Instruction::Unkr(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?, params[4].parse()?)
            }
            "addi" => {
                ensure!(params.len() == 3, "Wrong number of parameters");
                Instruction::Addi(params[0].parse()?, params[1].parse()?, params[2].parse()?)

            }
            &_ => bail!("Unknown instruction: {}", line)
        })
    }

}

pub trait Assembler {
    type Err;

    fn current_address(&self) -> u32;

    fn label(&mut self, name: &str, address: u32) -> Result<(), Self::Err>;

    fn lookup(&self, name: &str) -> Result<u32, Self::Err>;

    fn emit(&mut self, bits: impl Bits) -> Result<(), Self::Err>;
}


impl Instruction {
    pub fn assemble<Asm: Assembler>(self, asm: &mut Asm) -> Result<(), Asm::Err> {
        use Instruction::*;

        match self {
            Label(lbl) => asm.label(&lbl, asm.current_address())?,
            Unki(op, rd, rs, simm) => asm.emit(op | rd | rs | simm)?,
            Unkr(op, rd, rs, rt, simm) => asm.emit(op | rd | rs | rt | simm)?,
            Addi(rd, rs, simm) => asm.emit(Opcode(0x00) | rd | rs | simm)?,
        }

        Ok(())
    }
}