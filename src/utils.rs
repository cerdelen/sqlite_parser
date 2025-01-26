use std::fmt;
use anyhow::{Result, bail};

#[derive(Debug)]
pub struct VarInt {
    pub val: u64,
    pub len: usize,
}

impl std::fmt::Display for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Varint {{ val: {}, len: {} }}", self.val, self.len)
    }
}

impl VarInt {
    pub fn from_mem(bytes: &[u8]) -> Result<Self> {
        let mut result = 0u64;

        for (i, &byte) in bytes.iter().enumerate() {
            let value = (byte & 0b0111_1111) as u64;

            result |= result << 7;
            result |= value << 0;

            if byte & 0b1000_0000 == 0 {
                return Ok(Self { val: result, len: i+1 });
            }

            if i >= 8 {
                bail!("Varint is too long");
            }
        }
        bail!("Incomplete varint")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let var = VarInt::from_mem(&[0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 1);

        let var = VarInt::from_mem(&[0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 127);

        let var = VarInt::from_mem(&[0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b111_1111111_1111);

        let var = VarInt::from_mem(&[0b1000_0001, 0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 2097151);

        let var = VarInt::from_mem(&[0b1000_0001, 0b1000_0001, 0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b111_1111111_1111111_1111111_1111);

        let var = VarInt::from_mem(&[0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001000_0001000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b111_1111111_1111111_1111111_1111111_1111);

        let var = VarInt::from_mem(&[0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001000_0001000_0001000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b111_1111111_1111111_1111111_1111111_1111111_1111);

        let var = VarInt::from_mem(&[0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001000_0001000_0001000_0001000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b111_1111111_1111111_1111111_1111111_1111111_1111111_1111);

        let var = VarInt::from_mem(&[0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001000_0001000_0001000_0001000_0001000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111);

        let var = VarInt::from_mem(&[0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b1000_0001, 0b0000_0001]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b000_0001000_0001000_0001000_0001000_0001000_0001000_0001000_0001000_0001);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_ok());
        assert_eq!(var.unwrap().val, 0b111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111);

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111]);
        assert!(var.is_err());

        let var = VarInt::from_mem(&[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111]);
        assert!(var.is_err());
    }
}
