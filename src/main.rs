use std::{collections::BTreeMap, io::Write, path::PathBuf};

use anyhow::Context;
use clap::Parser;

pub mod fields;
//use fields::{Bits, Opcode, Rd, Rs, Rt, Uimm, Simm};
use fields::Bits;

pub mod instruction;
use instruction::Instruction;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to assemble
    input: PathBuf,

    /// Output file
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
    assembler.assemble(&input)?;
    assembler.output.clear();
    assembler.assemble(&input)?;

    println!("labels: {:?}", assembler.labels);

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

    fn assemble(&mut self, source: &str) -> Result<(), anyhow::Error> {
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with("#") || line == "" {
                continue;
            }
            line.parse::<Instruction>()?.assemble(self)?;
        }

        Ok(())
    }
}

impl instruction::Assembler for Assembler {
    type Err = anyhow::Error;

    fn current_address(&self) -> u32 {
        self.base_addr + self.output.len() as u32
    }

    fn label(&mut self, name: &str, address: u32) -> Result<(), Self::Err> {
        self.labels.insert(name.to_string(), address);
        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<u32, Self::Err> {
        Ok(*self.labels.get(name).unwrap_or(&0u32))
    }

    fn emit(&mut self, bits: impl Bits) -> Result<(), Self::Err> {
        self.output.extend_from_slice(&bits.bits().to_be_bytes());

        Ok(())
    }
}