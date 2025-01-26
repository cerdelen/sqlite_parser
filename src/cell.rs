use crate::utils::VarInt;
// use core::{panic;
use std::{fmt, usize};
use crate::page::PageType;
use anyhow::{bail, Ok, Result};

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cell {{")?;
        writeln!(f, "\tsize_record: {}", self.size_record)?;
        writeln!(f, "\trowId: {:?}", self.rowid)?;
        writeln!(f, "\tcontent: {:#?}", self.content)?;
    // pub content: Content,
        writeln!(f, "}}")
    }
}

#[derive(Debug)]
pub enum Record {
    Null,
    I8(i8),
    I16(i16),
    I24(i32),
    I32(i32),
    I48(i64),
    I64(i64),
    F64(f64),
    Val0,
    Val1,
    Reserved,
    Blob,
    String(String),
}

impl Record {
    fn mem_size(&self) -> usize {
        match self {
            Record::Null => 0,
            Record::I8(_) => 1,
            Record::I16(_) => 2,
            Record::I24(_) => 3,
            Record::I32(_) => 4,
            Record::I48(_) => 6,
            Record::I64(_) => 8,
            Record::F64(_) => 8,
            Record::Val0 => 0,
            Record::Val1 => 0,
            Record::Reserved => panic!("Should never encounter Reserved Record"),
            Record::Blob => todo!(),
            Record::String(s) => s.len(),
        }
    }

    fn new(bytes: &[u8], record_type: &VarInt) -> Result<Self> {
        let res = match record_type.val {
            0 => Self::Null,
            1 => {
                if bytes.len() < 1 {
                    return bail!("expect I8 but buffer only size of {}", bytes.len());
                };
                Self::I8(bytes[0] as i8)
            },
            2 => {
                if bytes.len() < 2 {
                    return bail!("expect I16 but buffer only size of {}", bytes.len());
                };
                Self::I16(i16::from_be_bytes([bytes[0], bytes[1]]))
            },
            3 => {
                if bytes.len() < 3 {
                    return bail!("expect I24 but buffer only size of {}", bytes.len());
                }
                Self::I24(i32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]))
            },
            4 => {
                if bytes.len() < 4 {
                    return bail!("expect I32 but buffer only size of {}", bytes.len());
                }
                Self::I32(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            },
            5 => {
                if bytes.len() < 6 {
                    return bail!("expect I48 but buffer only size of {}", bytes.len());
                }
                Self::I48(i64::from_be_bytes([0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]))
            },
            6 => {
                if bytes.len() < 8 {
                    return bail!("expect I64 but buffer only size of {}", bytes.len());
                }
                Self::I64(i64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
            },
            7 => {
                if bytes.len() < 8 {
                    return bail!("expect F64 but buffer only size of {}", bytes.len());
                }
                Self::F64(f64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
            },
            8 => Self::Val0,
            9 => Self::Val1,
            10 | 11 => Self::Reserved,
            val => {
                match val % 2 {
                    1 => {
                        let str_len = ((val - 13) / 2) as usize;
                        if bytes.len() < str_len {
                            return bail!("expected String of size {}, but buffer only size of {}", ((val - 13) / 2) as usize, bytes.len());
                        }
                        let s = String::from_utf8_lossy(&bytes[..((val - 13) / 2) as usize]);
                        Self::String(s.to_string())
                    },
                    0 => {
                        if bytes.len() < ((val - 12) / 2) as usize{
                            return bail!("expected Blob of size {}, but buffer only size of {}", ((val - 12) / 2) as usize, bytes.len());
                        }
                        Self::Blob
                    },
                    err => panic!("modulo 2 of {} returned neither 0 nor 1: {}", val, err),
                }
            },
        };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct Content {
    header_size: VarInt,
    schema_type: Record,
    schema_name: Record,
    schema_tbl_name: Record,
    schema_rootpage: Record,
    schema_sql: Record,
}

impl Content {
    pub fn is_table(&self) -> bool {
        if let Record::String(s) = &self.schema_type {
            if s == "table" {
                return true
            }
        }
        false
    }

    pub fn get_table_name(&self) -> Result<&str> {
        if self.is_table() {
            if let Record::String(s) = &self.schema_tbl_name {
                return Ok(s)
            }
        }
        bail!("not a table or not a string type")
    }

    fn new(bytes: &[u8]) -> Result<Self>{
        let mut ind: usize = 0;
        let header_size = VarInt::from_mem(&bytes[..10])?;
        ind += header_size.len;
        let schema_type_size = VarInt::from_mem(&bytes[ind..ind+10])?;
        ind += schema_type_size.len;
        let schema_name_size = VarInt::from_mem(&bytes[ind..ind+10])?;
        ind += schema_name_size.len;
        let table_name_size = VarInt::from_mem(&bytes[ind..ind+10])?;
        ind += table_name_size.len;
        let rootpage_type = VarInt::from_mem(&bytes[ind..ind+10])?;
        ind += rootpage_type.len;
        let sql_size = VarInt::from_mem(&bytes[ind..ind+10])?;
        ind += sql_size.len;

        // println!("before parsing records");
        // println!("header_size: {}", header_size);
        // println!("schema_type_size: {}", schema_type_size);
        // println!("schema_name_size: {}", schema_name_size);
        // println!("table_name_size: {}", table_name_size);
        // println!("rootpage_type: {}", rootpage_type);
        // println!("sql_size: {}", sql_size);

        let schema_type = Record::new(&bytes[ind..], &schema_type_size)?;
        ind += schema_type.mem_size();
        let schema_name = Record::new(&bytes[ind..], &schema_name_size)?;
        ind += schema_name.mem_size();
        let schema_tbl_name = Record::new(&bytes[ind..], &table_name_size)?;
        ind += schema_tbl_name.mem_size();
        let schema_rootpage = Record::new(&bytes[ind..], &rootpage_type)?;
        ind += schema_rootpage.mem_size();
        let schema_sql = Record::new(&bytes[ind..], &sql_size)?;
        Ok(
            Self {
                header_size,
                schema_type,
                schema_name,
                schema_tbl_name,
                schema_rootpage,
                schema_sql,
            }
        )
    }
}

pub struct Cell {
    size_record: VarInt,
    pub rowid: VarInt,
    pub content: Content,
    // pub record: Record,
}

impl Cell {
    pub fn record_size(&self) -> usize {
        self.size_record.val as usize
    }

    pub fn cell_size(&self) -> usize {
        (self.size_record.val as usize) + self.rowid.len + self.size_record.len
    }

    pub fn new(bytes: &[u8], page_type: &PageType) -> Result<Self> {
        match page_type {
            PageType::LeafTable => {
                // there can be overflow for cell spillage
                let size_record = VarInt::from_mem(&bytes[..9])?;
                let rowid = VarInt::from_mem(&bytes[size_record.len..size_record.len + 9])?;

                let content = Content::new(&bytes[&size_record.len + &rowid.len..])?;

                return Ok(Self { size_record, rowid, content });
            },
            _ => todo!(),
            // PageType::InteriorTable => todo!(),
            // PageType::LeafIndex => todo!(),
            // PageType::InteriorIndex => todo!(),
        };
    }
}
