use std::collections::{btree_map::Entry, BTreeMap};

use anyhow::{bail, ensure};

use crate::{fields::Bits, instructions::{Assembler, Instruction}};

pub fn assemble(base_addr: u32, source: &str) -> anyhow::Result<(Vec<u8>, BTreeMap<String, u32>)> {
    let instructions = Instruction::parse(source)?;

    let mut label_assembler = LabelAssembler::new(base_addr);
    label_assembler.assemble(&instructions)?;

    let mut output_assembler = OutputAssembler::new(base_addr, label_assembler.labels);
    output_assembler.assemble(&instructions)?;

    Ok((output_assembler.output, output_assembler.labels))
}

pub fn assemble_template(base_addr: u32, template: &str, parameters: &BTreeMap<String, u64>) -> anyhow::Result<(Vec<u8>, BTreeMap<String, u32>)> {
    let mut ctx = tera::Context::new();
    for (k, v) in parameters.iter() {
        ctx.insert(k, v);
    }
    let source = tera::Tera::one_off(&template, &ctx, false)?;
    let (code, labels) = assemble(base_addr, &source)?;
    Ok((code, labels))
}

pub struct LabelAssembler {
    base_addr: u32,
    labels: BTreeMap<String, u32>,
    offset: u32,
}

impl LabelAssembler {
    pub fn new(base_addr: u32) -> Self {
        Self {
            base_addr,
            labels: Default::default(),
            offset: Default::default(),
        }
    }
}

impl Assembler for LabelAssembler {
    type Err = anyhow::Error;

    fn current_address(&self) -> u32 {
        return self.base_addr + self.offset;
    }

    fn label(&mut self, name: &str, address: u32) -> Result<(), Self::Err> {
        if let Entry::Vacant(entry) = self.labels.entry(name.to_string()) {
            entry.insert(address);
        } else {
            bail!("label already defined");
        }
        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<u32, Self::Err> {
        Ok(self.labels.get(name).unwrap_or(&0xffffffff).clone())
    }

    fn emit(&mut self, _bits: impl Bits) -> Result<(), Self::Err> {
        self.offset += 4;
        Ok(())
    }
}

pub struct OutputAssembler {
    base_addr: u32,
    labels: BTreeMap<String, u32>,
    output: Vec<u8>,
}

impl OutputAssembler {
    pub fn new(base_addr: u32, labels: BTreeMap<String, u32>) -> Self {
        Self {
            base_addr,
            labels: labels,
            output: Default::default(),
        }
    }
}

impl Assembler for OutputAssembler {
    type Err = anyhow::Error;

    fn current_address(&self) -> u32 {
        self.base_addr + self.output.len() as u32
    }

    fn label(&mut self, name: &str, address: u32) -> Result<(), Self::Err> {
        if let Some(label_address) = self.labels.get(name) {
            ensure!(*label_address == address, "Label redefined");
        }
        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<u32, Self::Err> {
        if let Some(address) = self.labels.get(name) {
            return Ok(*address);
        }
        bail!("label undefined");
    }

    fn emit(&mut self, bits: impl Bits) -> Result<(), Self::Err> {
        self.output.extend_from_slice(&bits.bits().to_be_bytes());

        Ok(())
    }
}