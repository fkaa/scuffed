#[derive(thiserror::Error, Debug)]
pub enum MkvError {
    #[error("Not enough data")]
    NotEnoughData,
    #[error("Unsupported variable integer size: {0}")]
    UnsupportedVint(u8),
}

fn vint(bytes: &[u8]) -> Result<(u8, u64), MkvError> {
    let byte = bytes[0];
    let len = byte.leading_zeros() as u8;

    if len > 8 || len == 0 {
        return Err(MkvError::UnsupportedVint(len));
    }

    if len as usize + 1 > bytes.len() {
        return Err(MkvError::NotEnoughData);
    }

    let mut value = byte as u64 & ((1 << (8 - (len + 1))) - 1) as u64;
    println!("{:08b}", value);
    println!("{:08b}", byte);
    println!("{:08b}", ((1 << (8 - (len + 1))) - 1));
    dbg!(len);
    for i in 0..len {
        value <<= 8;
        value |= bytes[i as usize + 1] as u64;
    }

    Ok((len + 1, value))
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case(&[0b1000_0010], 1, 2)]
    #[test_case(&[0b0100_0000, 0b0000_0010], 2, 2)]
    #[test_case(&[0b0010_0000, 0b0000_0000, 0b0000_0010], 3, 2)]
    #[test_case(&[0b0001_0000, 0b0000_0000, 0b0000_0000, 0b0000_0010], 4, 2)]
    fn vint(bytes: &[u8], len: u8, expected: u64) {
        let value = super::vint(bytes);

        assert_eq!(Some((len, expected)), value);
    }

}

