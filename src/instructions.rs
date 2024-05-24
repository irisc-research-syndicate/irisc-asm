use core::str::FromStr;

use anyhow::{bail, ensure, Context};

use crate::fields::{Bits, Funct, Jmpop, Label, Off14, Off9, Opcode, Rd, Reg, Rs, Rt, Simm, StoreOff16, Uimm};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Label(Label),
    Unki(Opcode, Rd, Rs, Uimm<16>),
    Unkr(Opcode, Rd, Rs, Rt, Uimm<11>),
    Addi(Rd, Rs, Simm<16>),
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
    Ldb(Rd, Rs, Simm<16>),
    Ldq(Rd, Rs, Off14),
    Lduw(Rd, Rs, Off14),
    Ldd(Rd, Rs, Off14),
    Ldlw(Rd, Rs, Off14),
    Stb(Rt, Rs, StoreOff16),
    Std(Rd, Rs, Rt, Off9),
    Stq(Rd, Rs, Rt, Off9),
}

fn check_indices<const N: usize>(indices: [usize; N]) {
    assert_eq!(indices, std::array::from_fn(|i| i));
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        use Instruction::*;

        let line = line.trim();
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, &""));
        let params = rest
            .trim()
            .split(',')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>();

        macro_rules! params {
            ($variant:ident $( ( $($index:expr),* $(,)? ) )?) => {{
                let indices = [$($($index),*)?];
                check_indices(indices);
                ensure!(params.len() == indices.len(), "Wrong number of parameters");
                $variant $((
                    $(
                        params[$index].parse()?
                    ),*
                ))?
            }};
        }

        Ok(match cmd {
            "lbl" => params!(Label(0)),
            "unk.i" => params!(Unki(0, 1, 2, 3)),
            "unk.r" => params!(Unkr(0, 1, 2, 3, 4)),
            "addi" => params!(Addi(0, 1, 2)),
            "jump" => params!(Jump(0)),
            "call" => params!(Call(0)),
            "set0" => params!(Set0(0, 1, 2)),
            "set1" => params!(Set1(0, 1, 2)),
            "set2" => params!(Set2(0, 1, 2)),
            "set3" => params!(Set3(0, 1, 2)),
            "set32" => params!(Set32(0, 1)),
            "set64" => params!(Set64(0, 1)),
            "alu.r" => params!(Alur(0, 1, 2, 3)),
            "add" => params!(Add(0, 1, 2)),
            "sub" => params!(Sub(0, 1, 2)),
            "subs" => params!(Subs(0, 1, 2)),
            "ret.d" => params!(Retd),
            "ld.b" => params!(Ldb(0, 1, 2)),
            "ld.q" => params!(Ldq(0, 1, 2)),
            "ld.uw" => params!(Lduw(0, 1, 2)),
            "ld.d" => params!(Ldd(0, 1, 2)),
            "ld.lw" => params!(Ldlw(0, 1, 2)),
            "st.b" => params!(Stb(0, 1, 2)),
            "st.d" => params!(Std(0, 1, 2, 3)),
            "st.q" => params!(Stq(0, 1, 2, 3)),
            _ => bail!("Unknown instruction: {}", line),
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
            Unki(op, rd, rs, uimm) => asm.emit(op | rd | rs | uimm)?,
            Unkr(op, rd, rs, rt, uimm) => asm.emit(op | rd | rs | rt | uimm)?,
            Addi(rd, rs, simm) => asm.emit(Opcode::fixed(0x00) | rd | rs | simm)?,
            Jump(lbl) => {
                let offset: i32 = (asm.lookup(&lbl.0)? as i32 - asm.current_address() as i32) >> 2;
                asm.emit(Opcode::fixed(0x25) | Jmpop::Jump | Simm::<24>::new(offset as i64).unwrap())?
            }
            Call(lbl) => {
                let offset: i32 = (asm.lookup(&lbl.0)? as i32 - asm.current_address() as i32) >> 2;
                asm.emit(Opcode::fixed(0x25) | Jmpop::Call | Simm::<24>::new(offset as i64).unwrap())?
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
            Add(rd, rs, rt) => {
                asm.emit(Opcode::fixed(0x3f) | rd | rs | rt | Funct::fixed(0x000))?
            }
            Sub(rd, rs, rt) => {
                asm.emit(Opcode::fixed(0x3f) | rd | rs | rt | Funct::fixed(0x004))?
            }
            Subs(rd, rs, rt) => {
                asm.emit(Opcode::fixed(0x3f) | rd | rs | rt | Funct::fixed(0x005))?
            }
            Retd => asm.emit(Opcode::fixed(0x3f) | Funct::fixed(0x02d))?,
            Ldb(rd, rs, simm16) => asm.emit(Opcode::fixed(0x18) | rd | rs | simm16)?,
            Ldq(rd, rs, off14) => asm.emit(Opcode::fixed(0x19) | rd | rs | off14 | Uimm::<2>(0))?,
            Lduw(rd, rs, off14) => {
                asm.emit(Opcode::fixed(0x19) | rd | rs | off14 | Uimm::<2>(1))?
            }
            Ldd(rd, rs, off14) => asm.emit(Opcode::fixed(0x19) | rd | rs | off14 | Uimm::<2>(2))?,
            Ldlw(rd, rs, off14) => {
                asm.emit(Opcode::fixed(0x19) | rd | rs | off14 | Uimm::<2>(3))?
            }
            Stb(rt, rs, stoff16) => asm.emit(Opcode::fixed(0x1a) | rs | rt | stoff16)?,
            Std(rd, rs, rt, off9) => {
                asm.emit(Opcode::fixed(0x1b) | rd | rs | rt | off9 | Uimm::<2>(2))?
            }
            Stq(rd, rs, rt, off9) => {
                asm.emit(Opcode::fixed(0x1e) | rd | rs | rt | off9 | Uimm::<2>(0))?
            }
        }

        Ok(())
    }

    pub fn parse(source: &str) -> Result<Vec<Self>, anyhow::Error> {
        source
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| {
                line.parse()
                    .with_context(|| format!("Bad instruction: {}", line))
            })
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
                Uimm(0x1234)
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
                Uimm(0x34)
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
        assert_eq!(instructions, vec![Instruction::Retd])
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
                    Uimm(0x1234)
                ),
                Instruction::Unkr(
                    Opcode::fixed(0x13),
                    "r5".parse().unwrap(),
                    "r0".parse().unwrap(),
                    "r7".parse().unwrap(),
                    Uimm(0x34)
                ),
                Instruction::Jump("foobar".parse().unwrap()),
                Instruction::Call("foobar".parse().unwrap()),
            ]
        );
    }
}
