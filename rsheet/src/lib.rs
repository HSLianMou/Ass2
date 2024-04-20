use rsheet_lib::connect::{Manager, Reader, Writer};
use rsheet_lib::replies::Reply;

use std::collections::HashMap;
use std::fmt::Debug;
use regex::Regex;
use std::error::Error;
use std::thread;
use std::sync::{Arc, Mutex};

use log::info;

use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::{CommandRunner, CellArgument};
use rsheet_lib::cells::{column_number_to_name, column_name_to_number};

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let data_base_matrix = Arc::new(Mutex::new(vec![vec![CellValue::None; 10]; 26]));

    loop {
        if let Ok((reader, writer)) = manager.accept_new_connection() {
            let db_matrix_clone = Arc::clone(&data_base_matrix);
            thread::spawn(move || {
                handle_connection(reader, writer, db_matrix_clone);
            });
        }
    }
}

fn handle_connection<R, W>(mut reader: R, mut writer: W, data_base_matrix: Arc<Mutex<Vec<Vec<CellValue>>>>)
where
    R: Reader,
    W: Writer,
{
    loop {
        let msg = match reader.read_message() {
            Ok(msg) => msg.trim().to_string(),
            Err(_) => break,
        };

        let parts: Vec<&str> = msg.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0];
        match cmd {
            "get" => {
                if let Some(key) = parts.get(1) {
                    if let Ok((col, row)) = split_key(key) {
                        let matrix = data_base_matrix.lock().unwrap();
                        if row < matrix.len() && col < matrix[row].len() {
                            let cell_value = &matrix[row][col];
                            let reply = match cell_value {
                                CellValue::Error(err) => Reply::Error(format!("{} = Error: '{}'", key, err)),
                                _ => Reply::Value(key.to_string(), cell_value.clone())
                            };
                            writer.write_message(reply).ok();
                        }
                    }
                }
            },
            "set" => {
                if parts.len() >= 3 {
                    let key = parts[1];
                    let expression = parts[2..].join(" ");
                    if expression.starts_with("sum") {
                        let range = &expression[4..expression.len()-1];
                        let matrix_arg = {
                            let matrix = data_base_matrix.lock().unwrap();
                            extract_matrix_for_sum(range, &matrix)
                        };
                        let result = CommandRunner::new(&format!("sum({})", range)).run(&HashMap::from([(range.to_string(), matrix_arg)]));
                        if let Ok((col, row)) = split_key(key) {
                            let mut matrix = data_base_matrix.lock().unwrap();
                            set_cell_in_matrix(&mut matrix, col, row, result.clone());
                        }
                        writer.write_message(Reply::Value(key.to_string(), result)).ok();
                    } else {
                        let result_expression = {
                            let matrix = data_base_matrix.lock().unwrap();
                            replaced_cells_expression(&expression, &matrix)
                        };
                        if let Ok(replaced_expression) = result_expression {
                            let result = CommandRunner::new(&replaced_expression).run(&HashMap::new());
                            if let Ok((col, row)) = split_key(key) {
                                let mut matrix = data_base_matrix.lock().unwrap();
                                set_cell_in_matrix(&mut matrix, col, row, result.clone());
                            }
                            // writer.write_message(Reply::Value(key.to_string(), result)).ok();
                        }
                    }
                } else {
                    writer.write_message(Reply::Error(String::from("Syntax error"))).ok();
                }
            },
            _ => writer.write_message(Reply::Error(String::from("Unknown command"))).ok().expect("REASON"),
        }
    }
}


// pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
// where
//     M: Manager,
// {
//     let (mut recv, mut send) = manager.accept_new_connection().unwrap();
//     // let mut data_base: HashMap<String, CellValue> = HashMap::new();
//     let mut data_base_matrix: Vec<Vec<CellValue>> = vec![vec![CellValue::None; 10]; 26];

//     loop {
//         info!("Just got message");
//         let mut msg = recv.read_message()?;
//         // send.write_message(Reply::Error(format!("{msg:?}")))?;
//         msg = msg.trim().to_string();
//         // println!("{msg:?}");

//         let parts: Vec<&str> = msg.split_whitespace().collect();
        
//         // let matrix = CellArgument::matrix;
//         match parts.get(0) {
//             Some(&"get") => {
//                 if !check_format(parts[1]) {
//                     send.write_message(Reply::Error(format!("Invalid key Provided")));
//                 } else if check_format(parts[1]) {
//                     if let Ok((col, row)) = split_key(parts[1]) {
//                         if row < data_base_matrix.len() && col < data_base_matrix[row].len() {
//                             let cell_value = &data_base_matrix[row][col];
//                             match cell_value {
//                                 CellValue::Error(err) => {
//                                     send.write_message(Reply::Error(format!("{} = Error: '{}'", parts[1], err)))
//                                 },
//                                 _ => {
//                                     send.write_message(Reply::Value(parts[1].to_string(), cell_value.clone()))
//                                 }
//                             }?;
//                         }    
//                     }
//                 }
//             },

//             Some(&"set") => {
//                 if parts.len() < 3 {
//                     send.write_message(Reply::Error(format!("Syntax error")));
//                 } else if !check_format(parts[1]) {
//                     send.write_message(Reply::Error(format!("Invalid Key Provided")));
//                 } else {
//                     let key = parts[1];
//                     let expression = parts[2..].join(" ");
//                     if expression.starts_with("sum"){
//                         let range = &expression[4..expression.len()-1];
//                         let matrix_arg = extract_matrix_for_sum(range, &data_base_matrix);
//                         // println!("{:?}", matrix_arg);
//                         let result = CommandRunner::new(&format!("sum({})", range)).run(&HashMap::from([(range.to_string(), matrix_arg)]));
//                         if let Ok((col, row)) = split_key(parts[1]) {
//                             set_cell_in_matrix(&mut data_base_matrix, col, row, result.clone());
//                         }
//                         send.write_message(Reply::Value(key.to_string(), result))?;
//                     } else {
//                         match replaced_cells_expression(&expression, &data_base_matrix) {
//                             Ok(replaced_expression) => {
//                                 let runner = CommandRunner::new(&replaced_expression);
//                                 let result = runner.run(&HashMap::new());
//                                 if let Ok((col, row)) = split_key(parts[1]) {
//                                     set_cell_in_matrix(&mut data_base_matrix, col, row, result.clone());
//                                 }
//                                 println!("{}", result);
//                             },
    
//                             Err(e) => {
//                                 send.write_message(Reply::Error(format!("Error")))?;
//                             }
//                         };
//                     }
                    
//                 }
//             },

//             _ => todo!()
//         }
//     }
// }

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

        Ok((col_num.try_into().unwrap(), row_num.saturating_sub(1)))
    } else {
        Err("Invalid cell reference format")
    }
}

fn set_cell_in_matrix(matrix: &mut Vec<Vec<CellValue>>, col: usize, row: usize, value: CellValue) {
    if row >= matrix.len() {
        matrix.resize_with(row + 1, Vec::new);
    }
    if col >= matrix[row].len() {
        matrix[row].resize(col + 1, CellValue::None);
    }

    matrix[row][col] = value;
}

fn replaced_cells_expression(expression: &str, data_base_matrix: &Vec<Vec<CellValue>>) -> Result<String, String>{
    let re = Regex::new(r"([A-Z]+[0-9]+)").unwrap();
    let mut replaced_expression = expression.to_string();

    for cap in re.captures_iter(expression) {
        let cell_ref = cap.get(0).unwrap().as_str();
        if let Ok((col, row)) = split_key(cell_ref) {
            if row < data_base_matrix.len() && col < data_base_matrix[row].len() {
                let cell_value = &data_base_matrix[row][col];
                match cell_value {
                    CellValue::Int(value) => {
                        replaced_expression = replaced_expression.replace(cell_ref, &value.to_string());
                    },
                    CellValue::None => {
                        replaced_expression = replaced_expression.replace(cell_ref, "0");
                    },
                    _ => {
                        return Err(format!("Error"));
                    }
                }
            } else {
                return Err(format!("Error"));
            }
        } else {
            return Err(format!("Error"));
        }
    }
    Ok(replaced_expression)
}
fn extract_matrix_for_sum(range: &str, matrix: &Vec<Vec<CellValue>>) -> CellArgument {
    let bounds = range.split('_').collect::<Vec<_>>();
    if bounds.len() == 2 {
        if let (Ok((start_col, start_row)), Ok((end_col, end_row))) = (split_key(bounds[0]), split_key(bounds[1])) {
            // println!("{},{},,{}{}",start_col,start_row,end_col,end_row);
            let mut values = vec![];
            let max_row = matrix.len();
            let max_col = if !matrix.is_empty() { matrix[0].len() } else { 0 };

            for row in start_row..=end_row {
                let mut row_values = vec![];
                if row < max_row {
                    for col in start_col..=end_col {
                        if col < max_col {
                            row_values.push(matrix[row][col].clone());
                        } else {
                            row_values.push(CellValue::None);
                        }
                    }
                    values.push(row_values);
                } else {
                    values.push(vec![CellValue::None; (end_col - start_col + 1).min(max_col)]);
                }
            }
            return CellArgument::Matrix(values);
        }
    }
    CellArgument::Matrix(vec![vec![CellValue::Error("Invalid range provided".to_string())]])
}



