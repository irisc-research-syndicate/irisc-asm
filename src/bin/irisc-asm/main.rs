use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use irisc_asm::assemble_template;
use irisc_asm::utils::{parse_parameter, cartesian_product};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Shellcode {
    #[serde_as(as = "serde_with::hex::Hex")]
    code: Vec<u8>,
    parameters: BTreeMap<String, u64>,
    labels: BTreeMap<String, u32>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input: PathBuf,

    output: PathBuf,

    #[arg(short, long, default_value_t = 0)]
    base_addr: u32,

    #[arg(short, long, value_parser = parse_parameter)]
    param: Vec<(String, Vec<u64>)>
}


fn main() -> Result<()> {
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