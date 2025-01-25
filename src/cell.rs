use crate::utils::VarInt;
use core::panic;
use std::{fmt, usize};
use crate::page::PageType;
use anyhow::{bail, Result};
use bytes::Buf;

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cell {{")?;
        writeln!(f, "\tsize_record: {}", self.size_record)?;
        writeln!(f, "\trowId: {:?}", self.rowid)?;
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
    String,
}

impl Record {
    fn new(bytes: &[u8], record_type: &VarInt) -> Result<Self> {
        if bytes.len() < 1 {
            println!("lol");
        }
        todo!();
        // todo bytes of ints are read wrongly ... i am reading too many bytes for example in i24
        // and i48
        let res = match record_type.val {
            0 => Self::Null,
            1 => {
                if bytes.len() < 1 {
                    return bail!("expect I8 but buffer only size of {}", bytes.len());
                };
                // bytes.get_i8();
                Self::I8(bytes[0] as i8)
            },
            2 => {
                if bytes.len() < 2 {
                    return bail!("expect I16 but buffer only size of {}", bytes.len());
                };
                Self::I16(i16::from_be_bytes([bytes[1], bytes[2]]))
            },
            3 => {
                if bytes.len() < 3 {
                    return bail!("expect I24 but buffer only size of {}", bytes.len());
                }
                Self::I24(i32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]))
            },
            4 => {
                if bytes.len() < 4 {
                    return bail!("expect I32 but buffer only size of {}", bytes.len());
                }
                Self::I32(i32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]))
            },
            5 => {
                if bytes.len() < 6 {
                    return bail!("expect I48 but buffer only size of {}", bytes.len());
                }
                Self::I48(i64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]))
            },
            6 => {
                if bytes.len() < 8 {
                    return bail!("expect I64 but buffer only size of {}", bytes.len());
                }
                Self::I64(i64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]))
            },
            7 => {
                if bytes.len() < 8 {
                    return bail!("expect F64 but buffer only size of {}", bytes.len());
                }
                Self::F64(f64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]))
            },
            8 => Self::Val0,
            9 => Self::Val1,
            10 | 11 => Self::Reserved,
            val => {
                match val % 2 {
                    1 => {
                        if bytes.len() < ((val - 13) / 2) as usize {
                            return bail!("expected String of size {}, but buffer only size of {}", ((val - 13) / 2) as usize, bytes.len());
                        }
                        Self::String
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
// 0	0	Value is a NULL.
// 1	1	Value is an 8-bit twos-complement integer.
// 2	2	Value is a big-endian 16-bit twos-complement integer.
// 3	3	Value is a big-endian 24-bit twos-complement integer.
// 4	4	Value is a big-endian 32-bit twos-complement integer.
// 5	6	Value is a big-endian 48-bit twos-complement integer.
// 6	8	Value is a big-endian 64-bit twos-complement integer.
// 7	8	Value is a big-endian IEEE 754-2008 64-bit floating point number.
// 8	0	Value is the integer 0. (Only available for schema format 4 and higher.)
// 9	0	Value is the integer 1. (Only available for schema format 4 and higher.)
// 10,11 	variable	Reserved for internal use. These serial type codes will never appear in a well-formed database file, but they might be used in transient and temporary database files that SQLite sometimes generates for its own use. The meanings of these codes can shift from one release of SQLite to the next.
// N≥12 and even 	(N-12)/2	Value is a BLOB that is (N-12)/2 bytes in length.
// N≥13 and odd 	(N-13)/2	Value is a string in the text encoding and (N-13)/2 bytes in length. The nul terminator is not stored.

}

#[derive(Debug)]
struct Content {
    header_size: VarInt,
    schema_type: Record,
    schema_name: Record,
    schema_tbl_name: Record,
    schema_rootpage: Record,
    schema_sql: Record,
}

impl Content {
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

        let schema_type = Record::new(&bytes[ind..], &schema_type_size)?;
        ind += schema_type_size.val as usize;
        let schema_name = Record::new(&bytes[ind..], &schema_name_size)?;
        ind += schema_name_size.val as usize;
        let schema_tbl_name = Record::new(&bytes[ind..], &table_name_size)?;
        ind += table_name_size.val as usize;
        let schema_rootpage = Record::new(&bytes[ind..], &rootpage_type)?;
        ind += rootpage_type.val as usize;
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
    pub size_record: VarInt,
    pub rowid: Option<VarInt>,
    pub content: Content,
    // pub record: Record,
}

impl Cell {
    pub fn new(bytes: &[u8], page_type: &PageType) -> Result<Self> {
        match page_type {
            PageType::LeafTable => {
                // there can be overflow for cell spillage
                let size_of_cell = VarInt::from_mem(&bytes[..9])?;
                let rowid = VarInt::from_mem(&bytes[size_of_cell.len..size_of_cell.len + 9])?;

                let content = Content::new(&bytes[&size_of_cell.len + &rowid.len..])?;

                println!("content: {:?}", content);

                return Ok(Self { size_record: size_of_cell, rowid: Some(rowid), content });
            },
            _ => todo!(),
            // PageType::InteriorTable => todo!(),
            // PageType::LeafIndex => todo!(),
            // PageType::InteriorIndex => todo!(),
        };
    }
}
