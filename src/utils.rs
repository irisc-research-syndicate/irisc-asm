use anyhow::{bail, Context, Result};

// parse everything from -2**63-1 to 2**64-1 into a u64
pub fn parse_number(number: &str) -> Result<u64> {
    if let Some(number) = number.strip_prefix('-') {
        if let Some(hex_number) = number.strip_prefix("0x") {
            Ok(-i64::from_str_radix(hex_number, 16)? as u64)
        } else {
            Ok(-number.parse::<i64>()? as u64)
        }
    } else if let Some(hex_number) = number.strip_prefix("0x") {
        Ok(u64::from_str_radix(hex_number, 16)?)
    } else {
        Ok(number.parse::<u64>()?)
    }
}

pub fn parse_ranges(s: &str) -> Result<Vec<u64>> {
    s.split(',').map(|value|
        match value {
            func_and_count if func_and_count.contains(':') => {
                let (func, count) = func_and_count.split_once(':').unwrap();
                let count: usize = count.parse()?;
                match func {
                    "rand8" => Ok((0..count).map(|_| rand::random::<u8>() as u64).collect()),
                    "rand16" => Ok((0..count).map(|_| rand::random::<u16>() as u64).collect()),
                    "rand32" => Ok((0..count).map(|_| rand::random::<u32>() as u64).collect()),
                    "rand64" => Ok((0..count).map(|_| rand::random::<u64>() as u64).collect()),
                    "bits" => Ok((0..count).map(|i| (1 << i) as u64).collect()),
                    _ => bail!(format!("No such function {}", func)),
                }
            }
            number_or_range => {
                if let Some((low, high)) = number_or_range.split_once("..") {
                    let low = parse_number(low).context(format!("Invalid number: {}", low))?;
                    let high = parse_number(high).context(format!("Invalid number: {}", high))?;
                    Ok((0..high.wrapping_sub(low)).map(|x| x.wrapping_add(low)).collect())
                } else {
                    let number = parse_number(number_or_range).context(format!("Invalid number: {}", number_or_range))?;
                    Ok(vec![number])
                }
            }
        }
    ).collect::<Result<Vec<_>>>().map(|values| values.concat())
}

pub fn parse_parameter(s: &str) -> Result<(String, Vec<u64>)> {
    let (key, ranges) = s.split_once('=').context("no '=' is argument")?;
    Ok((key.to_string(), parse_ranges(ranges).context(format!("Invalid ranges: {}", ranges))?))
}

pub fn cartesian_product<K: Clone, V: Clone>(sets: Vec<(K, Vec<V>)>) -> Vec<Vec<(K, V)>> {
    if let Some(((k, set), rest)) = sets.split_first() {
        set.iter()
            .flat_map(|v| {
                cartesian_product(rest.to_vec()).into_iter().map(|mut row| {
                    row.push((k.clone(), v.clone()));
                    row
                })
            })
            .collect()
    } else {
        vec![vec![]]
    }
}
