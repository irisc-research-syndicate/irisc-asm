use std::{collections::BTreeMap, io::Write, path::PathBuf, str::FromStr};

use anyhow::Context;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to assemble
    #[arg(short, long)]
    input: PathBuf,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,

    #[arg(short, long, default_value_t = 0)]
    base_addr: u32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let input = std::fs::read_to_string(&args.input)?;
    let mut output = std::fs::File::create(&args.output)?;

    let mut assembler = Assembler::new(args.base_addr);

    // Generate labels
    assembler.parse(&input);
    assembler.output.clear();

    println!("labels: {:?}", assembler.labels);

    // Generate output
    assembler.parse(&input);

    output
        .write_all(&assembler.output)
        .context("could not write output")?;

    Ok(())
}

pub struct Assembler {
    base_addr: u32,
    labels: BTreeMap<String, u32>,
    output: Vec<u8>,
}

impl Assembler {
    pub fn new(base_addr: u32) -> Self {
        Self {
            labels: Default::default(),
            output: Default::default(),
            base_addr,
        }
    }

    pub fn parse(&mut self, lines: &str) {
        for line in lines.lines() {
            let line = line.trim();
            let Some((cmd, rest)) = line.split_once(' ') else {
                continue;
            };

            let rest = rest.trim();

            match cmd {
                "lbl" => {
                    self.labels
                        .insert(rest.to_owned(), self.base_addr + self.output.len() as u32);
                }
                _ => {
                    let params = rest.split(',').map(|p| p.trim()).collect::<Vec<_>>();
                    self.assemble_opcode(cmd, &params);
                }
            };
        }
    }

    pub fn assemble_opcode(&mut self, opcode: &str, params: &[&str]) {
        let res: u32 = match opcode {
            "addi" => {
                let op = 0x00 << 26;
                let rd: Register = params[0].parse().unwrap();
                let rs: Register = params[1].parse().unwrap();
                let imm: u32 = parse_immediate(params[2], true, 16);
                op | rd.as_rd() | rs.as_rs() | imm
            }
            _ => panic!("Invalid opcode {opcode}"),
        };

        self.output.extend_from_slice(&res.to_be_bytes());
    }
}

pub struct Register(u32);

impl Register {
    fn as_rd(&self) -> u32 {
        self.0 << 16
    }
    fn as_rs(&self) -> u32 {
        self.0 << 21
    }
    fn as_rt(&self) -> u32 {
        self.0 << 11
    }
}

fn parse_immediate(s: &str, signed: bool, bits: i32) -> u32 {
    let (s, is_negative) = if let Some(s) = s.strip_prefix('-') {
        (s, true)
    } else {
        (s, false)
    };

    if !signed && is_negative {
        panic!("encountered negative unsigned immediate");
    }

    let res = if let Some(s) = s.strip_prefix("0x") {
        i32::from_str_radix(s, 16).unwrap()
    } else {
        s.parse().unwrap()
    };

    (res & ((1i32 << bits) - 1)) as u32
}

#[derive(Debug)]
pub struct ParseRegisterError;

impl FromStr for Register {
    type Err = ParseRegisterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, rest) = s.split_at(1);
        if start != "r" {
            return Err(ParseRegisterError);
        }

        let num: u32 = rest.parse().map_err(|_| ParseRegisterError)?;
        if num > 31 {
            return Err(ParseRegisterError);
        }

        Ok(Self(num))
    }
}
