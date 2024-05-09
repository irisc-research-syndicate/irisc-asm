use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub mod fields;

pub mod instruction;

pub mod assembler;
use assembler::assemble_template;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Shellcode {
    #[serde_as(as = "serde_with::hex::Hex")]
    code: Vec<u8>,
    parameters: BTreeMap<String, u64>,
    labels: BTreeMap<String, u32>,
}

// parse everything from -2**63-1 to 2**64-1 into a u64
fn parse_number(number: &str) -> Result<u64> {
    if let Some(number) = number.strip_prefix("-") {
        if let Some(hex_number) = number.strip_prefix("0x") {
            Ok(-i64::from_str_radix(hex_number, 16)? as u64)
        } else {
            Ok(-i64::from_str_radix(number, 10)? as u64)
        }
    } else if let Some(hex_number) = number.strip_prefix("0x") {
        Ok(u64::from_str_radix(hex_number, 16)?)
    } else {
        Ok(number.parse()?)
    }
}

fn parse_parameter(s: &str) -> Result<(String, Vec<u64>)> {
    let (key, val) = s.split_once('=').context("no '=' is argument")?;
    let mut values = vec![];
    for value in val.split(",") {
        values.extend(
            match value {
                "rand8" => vec![rand::random::<u8>() as u64],
                "rand16" => vec![rand::random::<u16>() as u64],
                "rand32" => vec![rand::random::<u32>() as u64],
                "rand64" => vec![rand::random::<u64>() as u64],
                number_or_range => {
                    if let Some((low, high)) = number_or_range.split_once('-') {
                        (parse_number(low)?..=parse_number(high)?).collect()
                    } else {
                        vec![parse_number(number_or_range)?]
                    }
                }
            }
        )
    }
    Ok((key.to_string(), values))
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to assemble
    input: PathBuf,

    /// Output file
    output: PathBuf,

    #[arg(short, long)]
    labels: Option<PathBuf>,

    #[arg(short, long, default_value_t = 0)]
    base_addr: u32,

    #[arg(short, long, value_parser = parse_parameter)]
    param: Vec<(String, Vec<u64>)>
}

pub fn cartesian_product<K: Clone, V: Clone>(sets: Vec<(K, Vec<V>)>) -> Vec<Vec<(K, V)>> {
    if let Some(((k, set), rest)) = sets.split_first() {
        set.into_iter().flat_map(|v|
            cartesian_product(rest.to_vec()).into_iter().map(|mut row| {
                row.push((k.clone(), v.clone()));
                row
            })
        ).collect()
    } else {
        vec![vec![]]
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let template = std::fs::read_to_string(&args.input)?;

    for parameters in cartesian_product(args.param).into_iter().map(BTreeMap::from_iter) {
        let (code, labels) = assemble_template(args.base_addr, &template, &parameters)?;
        println!("{}", serde_json::to_string(&Shellcode {
            parameters,
            code,
            labels,
        })?);
    }

    Ok(())
}