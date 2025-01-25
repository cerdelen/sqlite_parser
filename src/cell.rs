use crate::utils::VarInt;
use std::fmt;
use crate::page::PageType;
use anyhow::Result;

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the write! macro to format the struct fields
        writeln!(f, "Cell(")?;
        writeln!(f, "\tsize_record: {}", self.size_record)?;
        writeln!(f, "\trowId: {:?}", self.rowid)?;
        writeln!(f, ")")
    }
}

struct Record {
    size_header: u64,
}

pub struct Cell {
    pub size_record: VarInt,
    pub rowid: Option<VarInt>,
    pub record: Record,
}

impl Cell {
    pub fn new(bytes: &[u8], page_type: &PageType) -> Result<Self> {
        match page_type {
            PageType::LeafTable => {
                // there can be overflow for cell spillage
                let size_of_cell = VarInt::from_mem(&bytes[..9])?;
                let rowid = VarInt::from_mem(&bytes[size_of_cell.len..size_of_cell.len + 9])?;

                return Ok(Self { size_record: size_of_cell, rowid: Some(rowid), record: Record{size_header: 0} });
            },
            _ => todo!(),
            // PageType::InteriorTable => todo!(),
            // PageType::LeafIndex => todo!(),
            // PageType::InteriorIndex => todo!(),
        };
    }
}
