use std::collections;
use std::vec;

use crate::cell::*;
use crate::db::DB;
use crate::page::*;

use anyhow::anyhow;
use regex::Regex;

use anyhow::Ok;
use anyhow::Result;

pub fn tables(db: &mut DB) -> Result<()> {
    let page = Page::new(db, 1)?;

    let tables = tables_from_page(&page)?;

    let mut tables_names: Vec<&str> = tables
        .iter()
        .filter_map(|table| {
            if let Content::TableCell(content) = &table.content {
                content.get_table_name().ok()
            } else {
                None
            }
        })
        .collect();

    tables_names.sort();

    for table_name in tables_names {
        print!("{} ", table_name);
    }
    println!("");

    Ok(())
}

pub fn db_info(db: &mut DB) -> Result<()> {
    let page = Page::new(db, 1)?;

    println!("database page size: {}", db.header.page_size);
    println!("number of tables: {}", page.cell_count);

    Ok(())
}

fn count_rows(db: &mut DB, table: &str) -> Result<()> {
    let page = Page::new(db, 1)?;

    let tables = tables_from_page(&page)?;

    if let Some(table) = find_table_by_name(&tables, table) {
        println!("{}", table);
        if let Content::TableCell(content) = &table.content {
            let table_page = Page::new(db, content.get_rootpage().get_numeric_val())?;

            if let PageType::LeafTable = table_page.page_type {
                println!("{}", table_page.cell_count);
            } else {
                println!("table is multipage table ... cant parse that yet");
            }
        }
    } else {
        println!("no such table: {}", table);
    }

    Ok(())
}

fn values_from_rows(db: &mut DB, page_ind: u64, ind: &Vec<usize>) -> Result<Vec<Vec<String>>> {
    if ind.len() == 0 {
        anyhow!("0 keys");
    }
    db.root_page(page_ind)?;
    let table_page = Page::new(db, page_ind)?;
    let contents = rows_from_page(&table_page)?;

    let mut ret = vec![];

    for content in contents {
        if let Content::RowCell(row) = content {
            let mut inner_ret = vec![];
            for c_ind in ind {
                if let Some(Record::String(s)) = row.row.get(c_ind.clone()) {
                    inner_ret.push(s.clone());
                }
            }
            ret.push(inner_ret);
        }
    }

    Ok(ret)
}

pub fn select_x_from_y(db: &mut DB, x: &[&str], y: &str) -> Result<()> {
    let mut keys = vec![];
    for key in x {
        keys.push(key.trim_matches(','));
    }

    let page = Page::new(db, 1)?;

    let tables = tables_from_page(&page)?;

    let mut column_ind = vec![];
    let mut rootpage_ind = None;

    if let Some(table) = find_table_by_name(&tables, y) {
        if let Content::TableCell(content) = &table.content {
            rootpage_ind = Some(content.get_rootpage().get_numeric_val());
            let table_regex = Regex::new(r#"(?i)CREATE\s+TABLE\s+"?(\w+)"?\s*\(([^;]+)\)"#).unwrap();
            let field_regex = Regex::new(r"(?m)^\s*(\w+)\s+[\w()]+").unwrap();

            // println!("table: {:?}", content);
            if let Some(caps) = table_regex.captures(content.get_sql().get_string_val()) {
                let fields_part = &caps[2];
                for (i, field_cap) in field_regex.captures_iter(fields_part).enumerate() {
                    if keys.contains(&&field_cap[1]) {
                        column_ind.push(i);
                    }
                }
            }
        }
    };

    if let Some(r_ind) = rootpage_ind {
        let vals = values_from_rows(db, r_ind, &column_ind)?;
        for row_val in vals {
            for (i, col_val) in row_val.iter().enumerate() {
                if i > 0 {
                    print!("|");
                }
                print!("{}", col_val);
            }
            println!();
        }
    }

    Ok(())
}

pub fn select(db: &mut DB, query: &[&str]) -> Result<()> {
    let mut ind = 0;
    for (i, token) in query.iter().enumerate() {
        if *token == "FROM" {
            ind = i;
            break;
        }
    };
    select_x_from_y(db, &query[0..ind], query.last().unwrap());
    Ok(())
}

pub fn sql_query(db: &mut DB, query: &str) -> Result<()> {
    let tokens: Vec<&str> = query.split(" ").collect();

    if tokens.len() < 4 {
        println!("syntax error, need more arguments");
    }

    if *tokens.get(1).unwrap() == "COUNT(*)" {
        return count_rows(db, tokens.last().unwrap());
    }

    select(db, &tokens[1..]);

    Ok(())
}

