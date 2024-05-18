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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("0").unwrap(), 0);
        assert_eq!(parse_number("1").unwrap(), 1);
        assert_eq!(parse_number("1234").unwrap(), 1234);
        assert_eq!(parse_number("0x8000000000000000").unwrap(), 0x8000000000000000);
        assert_eq!(parse_number("0xffffffffffffffff").unwrap(), 0xffffffffffffffff);

        assert_eq!(parse_number("-0").unwrap(), 0);
        assert_eq!(parse_number("-1").unwrap(), 0xffffffffffffffff);
        assert_eq!(parse_number("-16").unwrap(), 0xfffffffffffffff0);
        assert_eq!(parse_number("-0x10").unwrap(), 0xfffffffffffffff0);
        assert_eq!(parse_number("-0x7fffffffffffffff").unwrap(), 0x8000000000000001);
        assert!(parse_number("-0x8000000000000000").is_err());

        assert_eq!(parse_number("-1").unwrap(), parse_number("0xffffffffffffffff").unwrap());
    }

    #[test]
    fn test_parse_ranges() {
        assert_eq!(parse_ranges("1,2,3").unwrap(), vec![1,2,3]);
        assert_eq!(parse_ranges("0..5").unwrap(), vec![0,1,2,3,4]);
        assert_eq!(parse_ranges("0..2,3..5").unwrap(), vec![0,1,3,4]);
        assert_eq!(parse_ranges("-1..2").unwrap(), vec![-1i64 as u64, 0, 1]);
        assert_eq!(parse_ranges("-10..10").unwrap(), (-10..10).map(|x| x as u64).collect::<Vec<_>>());
        assert_eq!(parse_ranges("0xfffffffffffffff0..0xffffffffffffffff").unwrap(), (-16..-1).map(|x| x as u64).collect::<Vec<_>>());
    }

    #[test]
    fn test_parse_ranges_random() {
        assert_eq!(parse_ranges("rand8:16").unwrap().len(), 16);
        assert_ne!(parse_ranges("rand8:16").unwrap(), vec![0; 16]);
        assert_eq!(parse_ranges("rand8:16").unwrap().into_iter().map(|x| x & 0xffffffffffffff00).collect::<Vec<_>>(), vec![0u64; 16]);

        assert_eq!(parse_ranges("rand16:16").unwrap().len(), 16);
        assert_ne!(parse_ranges("rand16:16").unwrap(), vec![0; 16]);
        assert_eq!(parse_ranges("rand16:16").unwrap().into_iter().map(|x| x & 0xffffffffffff0000).collect::<Vec<_>>(), vec![0u64; 16]);

        assert_eq!(parse_ranges("rand32:16").unwrap().len(), 16);
        assert_ne!(parse_ranges("rand32:16").unwrap(), vec![0; 16]);
        assert_eq!(parse_ranges("rand32:16").unwrap().into_iter().map(|x| x & 0xffffffff00000000).collect::<Vec<_>>(), vec![0u64; 16]);

        assert_eq!(parse_ranges("rand64:16").unwrap().len(), 16);
        assert_ne!(parse_ranges("rand64:16").unwrap(), vec![0; 16]);
    }

    #[test]
    fn test_parse_ranges_bits() {
        assert_eq!(parse_ranges("bits:16").unwrap(), vec![1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768]);
    }

    #[test]
    fn test_parse_parameter() {
        assert_eq!(parse_parameter("r5=1,2,3,10..20,-10..-5,0xfedcba9876543210").unwrap(), (
            "r5".to_string(),
            vec![
                1,2,3,
                10,11,12,13,14,15,16,17,18,19,
                -10i64 as u64, -9i64 as u64, -8i64 as u64, -7i64 as u64, -6i64 as u64,
                0xfedcba9876543210,
            ]
        ));
    }
}
