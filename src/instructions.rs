use core::str::FromStr;

use anyhow::{bail, ensure, Context};

use crate::fields::{Bits, Jmpop, Label, Opcode, Rd, Reg, Rs, Rt, Simm, Uimm, Funct, Off9};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Set32(Rd, Uimm<32>),
    Set64(Rd, Uimm<64>),
    Alur(Funct, Rd, Rs, Rt),
    Add(Rd, Rs, Rt),
    Sub(Rd, Rs, Rt),
    Subs(Rd, Rs, Rt),
    Retd,
    Ldd(Rd, Rs, Rt, Off9),
    Std(Rd, Rs, Rt, Off9),
    Stq(Rd, Rs, Rt, Off9),
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        use Instruction::*;

        let line = line.trim();
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, &""));
        let params = rest.trim().split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).collect::<Vec<_>>();

        Ok(match cmd {
            "lbl" => {
                ensure!(params.len() == 1, "Wrong number of parameters");
                Label(params[0].parse()?)
            }
            "unk.i" => {
                ensure!(params.len() == 4, "Wrong number of parameters");
                Unki(
                    params[0].parse()?,
                    params[1].parse()?,
                    params[2].parse()?,
                    params[3].parse()?,
                )
            }
            "unk.r" => {
                ensure!(params.len() == 5, "Wrong number of parameters");
                Unkr(
                    params[0].parse()?,
                    params[1].parse()?,
                    params[2].parse()?,
                    params[3].parse()?,
                    params[4].parse()?,
                )
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
            "set32" => {
                ensure!(params.len() == 2, "Wrong number of parameters");
                Set32(params[0].parse()?, params[1].parse()?)
            }
            "set64" => {
                ensure!(params.len() == 2, "Wrong number of parameters");
                Set64(params[0].parse()?, params[1].parse()?)
            }
            "alu.r" => {
                ensure!(params.len() == 4, "Wrong number of parameters");
                Alur(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?)
            }
            "add" => {
                ensure!(params.len() == 3, "Wrong number of parameters");
                Add(params[0].parse()?, params[1].parse()?, params[2].parse()?)
            }
            "sub" => {
                ensure!(params.len() == 3, "Wrong number of parameters");
                Sub(params[0].parse()?, params[1].parse()?, params[2].parse()?)
            }
            "subs" => {
                ensure!(params.len() == 3, "Wrong number of parameters");
                Subs(params[0].parse()?, params[1].parse()?, params[2].parse()?)
            }
            "ret.d" => {
                ensure!(params.len() == 0, "Wrong number of parameters");
                Retd
            }
            "ld.d" => {
                ensure!(params.len() == 4, "Wrong number of parameters");
                Ldd(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?)
            }
            "st.d" => {
                ensure!(params.len() == 4, "Wrong number of parameters");
                Std(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?)
            }
            "st.q" => {
                ensure!(params.len() == 4, "Wrong number of parameters");
                Stq(params[0].parse()?, params[1].parse()?, params[2].parse()?, params[3].parse()?)
            }
            &_ => bail!("Unknown instruction: {}", line),
        })
    }
}

pub trait Assembler: Sized {
    type Err;

    fn current_address(&self) -> u32;

    fn label(&mut self, name: &str, address: u32) -> Result<(), Self::Err>;

    fn lookup(&self, name: &str) -> Result<u32, Self::Err>;

    fn emit(&mut self, bits: impl Bits) -> Result<(), Self::Err>;

    fn assemble(&mut self, instructions: &[Instruction]) -> Result<(), Self::Err> {
        for instruction in instructions {
            instruction.assemble(self)?
        }
        Ok(())
    }
}

impl Instruction {
    pub fn assemble<Asm: Assembler>(&self, asm: &mut Asm) -> Result<(), Asm::Err> {
        use Instruction::*;

        match self.clone() {
            Label(lbl) => asm.label(&lbl.0, asm.current_address())?,
            Unki(op, rd, rs, simm) => asm.emit(op | rd | rs | simm)?,
            Unkr(op, rd, rs, rt, simm) => asm.emit(op | rd | rs | rt | simm)?,
            Addi(rd, rs, simm) => asm.emit(Opcode::fixed(0x00) | rd | rs | simm)?,
            Jump(lbl) => JumpInst(Jmpop::Jump, lbl).assemble(asm)?,
            Call(lbl) => JumpInst(Jmpop::Call, lbl).assemble(asm)?,
            JumpInst(jmpop, lbl) => {
                let offset: i32 = (asm.lookup(&lbl.0)? as i32 - asm.current_address() as i32) >> 2;
                asm.emit(Opcode::fixed(0x25) | jmpop | Simm::<24>::new(offset as i64).unwrap())?;
            }
            Set0(rd, rs, uimm) => asm.emit(Opcode::fixed(0x06) | rd | rs | uimm)?,
            Set1(rd, rs, uimm) => asm.emit(Opcode::fixed(0x07) | rd | rs | uimm)?,
            Set3(rd, rs, uimm) => asm.emit(Opcode::fixed(0x08) | rd | rs | uimm)?,
            Set2(rd, rs, uimm) => asm.emit(Opcode::fixed(0x09) | rd | rs | uimm)?,
            Set64(rd, uimm) => {
                Set0(rd, Rs(Reg(0)), Uimm((uimm.0 >> 48) & 0xffff)).assemble(asm)?;
                Set1(rd, Rs(rd.0), Uimm((uimm.0 >> 32) & 0xffff)).assemble(asm)?;
                Set2(rd, Rs(rd.0), Uimm((uimm.0 >> 16) & 0xffff)).assemble(asm)?;
                Set3(rd, Rs(rd.0), Uimm(uimm.0 & 0xffff)).assemble(asm)?;
            }
            Set32(rd, uimm) => {
                Set2(rd, Rs(Reg(0)), Uimm((uimm.0 >> 16) & 0xffff)).assemble(asm)?;
                Set3(rd, Rs(rd.0), Uimm(uimm.0 & 0xffff)).assemble(asm)?;
            }
            Alur(funct, rd, rs, rt) => asm.emit(Opcode::fixed(0x3f) | rd | rs | rt | funct)?,
            Add(rd, rs, rt) => asm.emit(Opcode::fixed(0x3f) | rd | rs | rt | Funct::fixed(0x000))?,
            Sub(rd, rs, rt) => asm.emit(Opcode::fixed(0x3f) | rd | rs | rt | Funct::fixed(0x004))?,
            Subs(rd, rs, rt) => asm.emit(Opcode::fixed(0x3f) | rd | rs | rt | Funct::fixed(0x005))?,
            Retd => asm.emit(Opcode::fixed(0x3f) | Funct::fixed(0x02d))?,
            Ldd(rd, rs,rt, off9) => asm.emit(Opcode::fixed(0x19) | rd | rs | rt | off9 | Uimm::<2>(2))?,
            Std(rd, rs,rt, off9) => asm.emit(Opcode::fixed(0x1b) | rd | rs | rt | off9 | Uimm::<2>(2))?,
            Stq(rd, rs,rt, off9) => asm.emit(Opcode::fixed(0x1e) | rd | rs | rt | off9 | Uimm::<2>(2))?,
        }

        Ok(())
    }

    pub fn parse(source: &str) -> Result<Vec<Self>, anyhow::Error> {
        source
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| line.parse().with_context(|| format!("Bad instruction: {}", line)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_parse_addi() {
        let instructions = Instruction::parse("addi r5, r0, 0x1234").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Addi(
                "r5".parse().unwrap(),
                "r0".parse().unwrap(),
                Simm::new(0x1234).unwrap()
            ),]
        );
    }

    #[test]
    fn instruction_parse_unki() {
        let instructions = Instruction::parse("unk.i 0x12, r5, r0, 0x1234").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Unki(
                Opcode::fixed(0x12),
                "r5".parse().unwrap(),
                "r0".parse().unwrap(),
                Simm::new(0x1234).unwrap()
            ),]
        );
    }

    #[test]
    fn instruction_parse_unkr() {
        let instructions = Instruction::parse("unk.r 0x12, r5, r0, r6, 0x34").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Unkr(
                Opcode::fixed(0x12),
                "r5".parse().unwrap(),
                "r0".parse().unwrap(),
                "r6".parse().unwrap(),
                Simm::new(0x34).unwrap()
            ),]
        );
    }

    #[test]
    fn instruction_parse_jump() {
        let instructions = Instruction::parse("jump foobar").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Jump("foobar".parse().unwrap()),]
        );
    }

    #[test]
    fn instruction_parse_call() {
        let instructions = Instruction::parse("call foobar").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Call("foobar".parse().unwrap()),]
        );
    }

    #[test]
    fn instruction_parse_set32() {
        let instructions = Instruction::parse("set32 r5, 0x12345678").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Set32("r5".parse().unwrap(), Uimm(0x12345678)),]
        );
    }

    #[test]
    fn instruction_parse_set64() {
        let instructions = Instruction::parse("set64 r5, 0x8765432112345678").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Set64(
                "r5".parse().unwrap(),
                Uimm(0x8765432112345678)
            ),]
        );
    }

    #[test]
    fn instruction_parse_retd() {
        let instructions = Instruction::parse("ret.d").unwrap();
        assert_eq!(
            instructions,
            vec![Instruction::Retd]
        )
    }

    #[test]
    fn instruction_parse_multiple() {
        let instructions = Instruction::parse(
            r#"
            addi r5, r0, 0x1234
            unk.i 0x13, r5, r0, 0x1234
            unk.r 0x13, r5, r0, r7, 0x34
            # this is a comment
            jump foobar

            call foobar
        "#,
        )
        .unwrap();
        assert_eq!(
            instructions,
            vec![
                Instruction::Addi(
                    "r5".parse().unwrap(),
                    "r0".parse().unwrap(),
                    Simm::new(0x1234).unwrap()
                ),
                Instruction::Unki(
                    Opcode::fixed(0x13),
                    "r5".parse().unwrap(),
                    "r0".parse().unwrap(),
                    Simm(0x1234)
                ),
                Instruction::Unkr(
                    Opcode::fixed(0x13),
                    "r5".parse().unwrap(),
                    "r0".parse().unwrap(),
                    "r7".parse().unwrap(),
                    Simm(0x34)
                ),
                Instruction::Jump("foobar".parse().unwrap()),
                Instruction::Call("foobar".parse().unwrap()),
            ]
        );
    }
}
