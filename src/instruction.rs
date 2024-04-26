use core::str::FromStr;

use anyhow::{bail, ensure};

use crate::fields::{Bits, Jmpop, Label, Opcode, Rd, Rs, Rt, Simm, Uimm};

pub enum Instruction {
    Label(Label),
    Unki(Opcode, Rd, Rs, Simm<16>),
    Unkr(Opcode, Rd, Rs, Rt, Simm<11>),
    Addi(Rd, Rs, Simm<16>),
    JumpInst(Jmpop, Label),
    Jump(Label),
    Call(Label),
    Set0(Rd, Rs, Uimm<16>),
    Set1(Rd, Rs, Uimm<16>),
    Set2(Rd, Rs, Uimm<16>),
    Set3(Rd, Rs, Uimm<16>),
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        use Instruction::*;

        let line = line.trim();
        let Some((cmd, rest)) = line.split_once(' ') else {
            bail!("Bad instruction: {}", line)
        };

        let params = rest.trim().split(',').map(|p| p.trim()).collect::<Vec<_>>();

        Ok(match cmd {
            "lbl" => {
                ensure!(params.len() == 1, "Wrong number of parameters");
                Label(params[0].parse()?)
            },
            "unk.i" => {
                ensure!(params.len() == 4, "Wrong number of parameters");
                Unki(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?)
            }
            "unk.r" => {
                ensure!(params.len() == 5, "Wrong number of parameters");
                Unkr(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?, params[4].parse()?)
            }
            "addi" => {
                ensure!(params.len() == 3, "Wrong number of parameters");
                Addi(params[0].parse()?, params[1].parse()?, params[2].parse()?)
            }
            "jump" => {
                ensure!(params.len() == 1, "Wrong number of parameters");
                Jump(params[0].parse()?)
            }
            "call" => {
                ensure!(params.len() == 1, "Wrong number of parameters");
                Call(params[0].parse()?)
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
            Label(lbl) => asm.label(&lbl.0, asm.current_address())?,
            Unki(op, rd, rs, simm) => asm.emit(op | rd | rs | simm)?,
            Unkr(op, rd, rs, rt, simm) => asm.emit(op | rd | rs | rt | simm)?,
            Addi(rd, rs, simm) => asm.emit(Opcode::fixed(0x00) | rd | rs | simm)?,
            Jump(lbl) => JumpInst(Jmpop::Jump, lbl).assemble(asm)?,
            Call(lbl) => JumpInst(Jmpop::Call, lbl).assemble(asm)?,
            JumpInst(jmpop, lbl) => {
                let offset: i32 = (asm.lookup(&lbl.0)? as i32 - asm.current_address() as i32) >> 2;
                asm.emit(Opcode::fixed(0x25) | jmpop | Simm::<24>::new(offset).unwrap())?;
            },
            Set0(rd, rs, uimm) => asm.emit(Opcode::fixed(0x06) | rd | rs | uimm)?,
            Set1(rd, rs, uimm) => asm.emit(Opcode::fixed(0x07) | rd | rs | uimm)?,
            Set3(rd, rs, uimm) => asm.emit(Opcode::fixed(0x08) | rd | rs | uimm)?,
            Set2(rd, rs, uimm) => asm.emit(Opcode::fixed(0x09) | rd | rs | uimm)?
        }

        Ok(())
    }
}