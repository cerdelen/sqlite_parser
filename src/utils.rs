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
        let mut shift = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            let value = (byte & 0b0111_1111) as u64;
            result |= value << shift;

            if byte & 0b1000_0000 == 0 {
                return Ok(Self { val: result, len: i+1 });
            }

            shift += 7;
            if shift >= 64 {
                bail!("Varint is too long");
            }
        }
        bail!("Incomplete varint")
    }
}
