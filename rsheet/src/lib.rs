use rsheet_lib::connect::{Manager, Reader, Writer};
use rsheet_lib::replies::Reply;

use std::collections::HashMap;
use std::fmt::Debug;
use regex::Regex;
use std::error::Error;

use log::info;

use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::{CommandRunner, CellArgument};
use rsheet_lib::cells::{column_number_to_name, column_name_to_number};

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let (mut recv, mut send) = manager.accept_new_connection().unwrap();
    let mut data_base: HashMap<String, CellValue> = HashMap::new();
    let mut data_base_matrix: Vec<Vec<CellValue>> = vec![vec![CellValue::None; 10]; 26];

    loop {
        info!("Just got message");
        let mut msg = recv.read_message()?;
        // send.write_message(Reply::Error(format!("{msg:?}")))?;
        msg = msg.trim().to_string();
        // println!("{msg:?}");

        let parts: Vec<&str> = msg.split_whitespace().collect();
        
        // let matrix = CellArgument::matrix;
        match parts.get(0) {
            Some(&"get") => {
                if !check_format(parts[1]) {
                    send.write_message(Reply::Error(format!("Invalid key Provided")));
                } else if check_format(parts[1]) {
                    if let Ok((col, row)) = split_key(parts[1]) {
                        if row < data_base_matrix.len() && col < data_base_matrix[row].len() {
                            let cell_value = &data_base_matrix[row][col];
                            match cell_value {
                                CellValue::Error(err) => {
                                    send.write_message(Reply::Error(format!("{} = Error: '{}'", parts[1], err)))
                                },
                                _ => {
                                    send.write_message(Reply::Value(parts[1].to_string(), cell_value.clone()))
                                }
                            }?;
                        }    
                    }
                }
                // if let Some(value) = parts.get(1).and_then(|&key| data_base.get(key)) {
                //     match value {
                //         CellValue::Error(err) => {
                //             send.write_message(Reply::Error(format!("{} = Error: '{}'", parts[1], err)))
                //         },
                //         _ => {
                //             send.write_message(Reply::Value(parts[1].to_string(), value.clone()))
                //         }
                //     }?;
                //     // println!("{:?}",value.clone());
                // } else {
                //     send.write_message(Reply::Error(format!("Variable not found")));
                // }
            },

            Some(&"set") => {
                if parts.len() < 3 {
                    send.write_message(Reply::Error(format!("Syntax error")));
                } else if !check_format(parts[1]) {
                    send.write_message(Reply::Error(format!("Invalid Key Provided")));
                } else {
                    let key = parts[1];
                    
                    let expression = parts[2..].join(" ");
                    let runner = CommandRunner::new(&expression);
                    let result = runner.run(&HashMap::new());
                    if let Ok((col, row)) = split_key(parts[1]) {
                        set_cell_in_matrix(&mut data_base_matrix, col, row, result.clone());
                    }
                    println!("{}", result);
                    data_base.insert(key.to_string().clone(), result.clone());
                }
            },

            None => todo!(),
            Some(&&_) => todo!()
        }
    }
}

fn check_format(key: &str) -> bool {
    let re = Regex::new(r"^([A-Z]+)(\d+)$").unwrap();

    if let Some(caps) = re.captures(key) {
        let column_part = caps.get(1).map_or("", |m| m.as_str());
        let row_part = caps.get(2).map_or("", |m| m.as_str());

        let column_valid = {
            let num = column_name_to_number(column_part);
            let generated_key = column_number_to_name(num);
            column_part == generated_key
        };

        let row_valid = row_part.parse::<u32>().map_or(false, |num| num > 0);

        column_valid && row_valid
    } else {
        false
    }
}

fn split_key(key: &str) -> Result<(usize, usize), &'static str> {
    let re = Regex::new(r"^([A-Z]+)(\d+)$").unwrap();

    if let Some(caps) = re.captures(key){
        let column_part = caps.get(1).map_or("", |m| m.as_str());
        let row_part = caps.get(2).map_or("", |m| m.as_str());

        let col_num = column_name_to_number(column_part);
        let row_num: usize = row_part.parse().map_err(|_| "Invalid row")?;

        Ok((col_num.saturating_sub(1).try_into().unwrap(), row_num.saturating_sub(1)))
    } else {
        Err("Invalid cell reference format")
    }
}

fn set_cell_in_matrix(matrix: &mut Vec<Vec<CellValue>>, col: usize, row: usize, value: CellValue) {
    // 确保矩阵的大小能够容纳新值
    if row >= matrix.len() {
        matrix.resize_with(row + 1, Vec::new);
    }
    if col >= matrix[row].len() {
        matrix[row].resize(col + 1, CellValue::None);
    }

    // 设置值
    matrix[row][col] = value;
}