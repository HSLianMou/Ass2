// use rsheet_lib::connect::{Manager, Reader, Writer};
// use rsheet_lib::replies::Reply;

// use std::collections::HashMap;
// // use std::fmt::Debug;
// use regex::Regex;
// use std::error::Error;
// use std::thread;
// use std::sync::{Arc, Mutex};

// // use log::info;

// use rsheet_lib::cell_value::CellValue;
// use rsheet_lib::command_runner::{CommandRunner, CellArgument};
// use rsheet_lib::cells::{column_number_to_name, column_name_to_number};

// pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
// where
//     M: Manager,
// {
//     let data_base_matrix = Arc::new(Mutex::new(vec![vec![CellValue::None; 10]; 26]));

//     loop {
//         if let Ok((reader, writer)) = manager.accept_new_connection() {
//             let db_matrix_clone = Arc::clone(&data_base_matrix);
//             thread::spawn(move || {
//                 handle_connection(reader, writer, db_matrix_clone);
//             });
//         }
//     }
//     // Ok(())
// }

// fn handle_connection<R, W>(mut reader: R, mut writer: W, data_base_matrix: Arc<Mutex<Vec<Vec<CellValue>>>>)
// where
//     R: Reader,
//     W: Writer,
// {
//     loop {
//         let msg = match reader.read_message() {
//             Ok(msg) => msg.trim().to_string(),
//             Err(_) => break,
//         };

//         let parts: Vec<&str> = msg.split_whitespace().collect();
//         if parts.is_empty() {
//             continue;
//         }

//         let cmd = parts[0];
//         match cmd {
//             "get" => {
//                 if !check_format(parts[1]) {
//                     let _ = writer.write_message(Reply::Error(format!("Invalid key Provided")));
//                 } else if check_format(parts[1]) {
//                     let key = parts[1];
//                     if let Ok((col, row)) = split_key(key) {
//                         let matrix = data_base_matrix.lock().unwrap();
//                         if row < matrix.len() && col < matrix[row].len() {
//                             let cell_value = &matrix[row][col];
//                             let reply = match cell_value {
//                                 CellValue::Error(err) => Reply::Error(format!("{} = Error: '{}'", key, err)),
//                                 _ => Reply::Value(key.to_string(), cell_value.clone())
//                             };
//                             writer.write_message(reply).ok();
//                         }
//                     }
//                 }
//             },
//             "set" => {
//                 if parts.len() < 3 {
//                     let _ = writer.write_message(Reply::Error(format!("Syntax error")));
//                 } else if !check_format(parts[1]) {
//                     let _ = writer.write_message(Reply::Error(format!("Invalid Key Provided")));
//                 } else {
//                     let key = parts[1];
//                     let expression = parts[2..].join(" ");
//                     if expression.starts_with("sum") {
//                         let range = &expression[4..expression.len()-1];
//                         let matrix_arg = {
//                             let matrix = data_base_matrix.lock().unwrap();
//                             extract_matrix_for_sum(range, &matrix)
//                         };
//                         let result = CommandRunner::new(&format!("sum({})", range)).run(&HashMap::from([(range.to_string(), matrix_arg)]));
//                         if let Ok((col, row)) = split_key(key) {
//                             let mut matrix = data_base_matrix.lock().unwrap();
//                             set_cell_in_matrix(&mut matrix, col, row, result.clone());
//                         }
//                         // writer.write_message(Reply::Value(key.to_string(), result)).ok();
//                     } else {
//                         let result_expression = {
//                             let matrix = data_base_matrix.lock().unwrap();
//                             replaced_cells_expression(&expression, &matrix)
//                         };
//                         if let Ok(replaced_expression) = result_expression {
//                             // println!("{}", &replaced_expression);
//                             if replaced_expression == "None" {
//                                 // let result = "None".to_string();
//                                 if let Ok((col, row)) = split_key(key) {
//                                     let mut matrix = data_base_matrix.lock().unwrap();
//                                     matrix[row][col] = CellValue::None;
//                                     // writer.write_message(Reply::Value(key.to_string(), CellValue::None)).ok();
//                                 }
//                             } else {
//                                 let result = CommandRunner::new(&replaced_expression).run(&HashMap::new());
//                                 if let Ok((col, row)) = split_key(key) {
//                                     let mut matrix = data_base_matrix.lock().unwrap();
//                                     set_cell_in_matrix(&mut matrix, col, row, result.clone());
//                                     // writer.write_message(Reply::Value(key.to_string(), result)).ok();
//                                 }
//                             }
//                             // writer.write_message(Reply::Value(key.to_string(), result)).ok();
//                         }
//                     }
//                 }
//             },
//             _ => writer.write_message(Reply::Error(String::from("Unknown command"))).ok().expect("REASON"),
//         }
//     }
// }



// fn check_format(key: &str) -> bool {
//     let re = Regex::new(r"^([A-Z]+)(\d+)$").unwrap();

//     if let Some(caps) = re.captures(key) {
//         let column_part = caps.get(1).map_or("", |m| m.as_str());
//         let row_part = caps.get(2).map_or("", |m| m.as_str());

//         let column_valid = {
//             let num = column_name_to_number(column_part);
//             let generated_key = column_number_to_name(num);
//             column_part == generated_key
//         };

//         let row_valid = row_part.parse::<u32>().map_or(false, |num| num > 0);

//         column_valid && row_valid
//     } else {
//         false
//     }
// }

// fn split_key(key: &str) -> Result<(usize, usize), &'static str> {
//     let re = Regex::new(r"^([A-Z]+)(\d+)$").unwrap();

//     if let Some(caps) = re.captures(key){
//         let column_part = caps.get(1).map_or("", |m| m.as_str());
//         let row_part = caps.get(2).map_or("", |m| m.as_str());

//         let col_num = column_name_to_number(column_part);
//         let row_num: usize = row_part.parse().map_err(|_| "Invalid row")?;

//         Ok((col_num.try_into().unwrap(), row_num.saturating_sub(1)))
//     } else {
//         Err("Invalid cell reference format")
//     }
// }

// fn set_cell_in_matrix(matrix: &mut Vec<Vec<CellValue>>, col: usize, row: usize, value: CellValue) {
//     if row >= matrix.len() {
//         matrix.resize_with(row + 1, Vec::new);
//     }
//     if col >= matrix[row].len() {
//         matrix[row].resize(col + 1, CellValue::None);
//     }

//     matrix[row][col] = value;
// }


// fn replaced_cells_expression(expression: &str, data_base_matrix: &Vec<Vec<CellValue>>) -> Result<String, String> {
//     let re = Regex::new(r"([A-Z]+[0-9]+)").unwrap();
//     let mut result_expression = String::with_capacity(expression.len());

//     let mut last_end = 0;
//     for cap in re.captures_iter(expression) {
//         let match_str = cap.get(0).unwrap().as_str();
//         let (col, row) = split_key(match_str).map_err(|_| "Invalid cell reference")?;
        
//         if row < data_base_matrix.len() && col < data_base_matrix[row].len() {
//             let value_str = match &data_base_matrix[row][col] {
//                 CellValue::Int(value) => value.to_string(),
//                 CellValue::None => "None".to_string(),
//                 _ => return Err("Error".to_string()),
//             };
            
//             result_expression.push_str(&expression[last_end..cap.get(0).unwrap().start()]);
//             result_expression.push_str(&value_str);
//             last_end = cap.get(0).unwrap().end();
//         }
//     }

//     result_expression.push_str(&expression[last_end..]);
//     Ok(result_expression)
// }

// fn extract_matrix_for_sum(range: &str, matrix: &Vec<Vec<CellValue>>) -> CellArgument {
//     let bounds = range.split('_').collect::<Vec<_>>();
//     if bounds.len() == 2 {
//         if let (Ok((start_col, start_row)), Ok((end_col, end_row))) = (split_key(bounds[0]), split_key(bounds[1])) {
//             // println!("{},{},,{}{}",start_col,start_row,end_col,end_row);
//             let mut values = vec![];
//             let max_row = matrix.len();
//             let max_col = if !matrix.is_empty() { matrix[0].len() } else { 0 };

//             for row in start_row..=end_row {
//                 let mut row_values = vec![];
//                 if row < max_row {
//                     for col in start_col..=end_col {
//                         if col < max_col {
//                             row_values.push(matrix[row][col].clone());
//                         } else {
//                             row_values.push(CellValue::None);
//                         }
//                     }
//                     values.push(row_values);
//                 } else {
//                     values.push(vec![CellValue::None; (end_col - start_col + 1).min(max_col)]);
//                 }
//             }
//             return CellArgument::Matrix(values);
//         }
//     }
//     CellArgument::Matrix(vec![vec![CellValue::Error("Invalid range provided".to_string())]])
// }



use rsheet_lib::connect::{Manager, Reader, Writer};
use rsheet_lib::replies::Reply;

use std::collections::HashMap;
// use std::fmt::Debug;
use regex::Regex;
use std::error::Error;
use std::thread;
use std::sync::{Arc, Mutex};

// use log::info;

use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::{CommandRunner, CellArgument};
use rsheet_lib::cells::{column_number_to_name, column_name_to_number};

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let data_base_matrix = Arc::new(Mutex::new(vec![vec![CellValue::None; 10]; 26]));
    let running = Arc::new(Mutex::new(true)); 

    loop {
        {
            let running_lock = running.lock().unwrap();
            if !*running_lock {
                break;
            }
        }
        if let Ok((reader, writer)) = manager.accept_new_connection() {
            let db_matrix_clone = Arc::clone(&data_base_matrix);
            let running_clone = Arc::clone(&running);
            let builder = thread::Builder::new();

            let handler = builder.spawn(move || {
                handle_connection(reader, writer, db_matrix_clone, running_clone);
            });
            if handler.is_err() {
                // eprintln!("Failed to create thread: {:?}", handler.err());
                break;
            }
        }
    }
    Ok(())
}

fn handle_connection<R, W>(mut reader: R, mut writer: W, data_base_matrix: Arc<Mutex<Vec<Vec<CellValue>>>>, running: Arc<Mutex<bool>>)
where
    R: Reader,
    W: Writer,
{
    loop {
        let msg = match reader.read_message() {
            Ok(msg) => msg.trim().to_string(),
            Err(_) => {
                *running.lock().unwrap() = false;
                break;
            },
        };

        let parts: Vec<&str> = msg.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0];
        match cmd {
            "get" => {
                if !check_format(parts[1]) {
                    let _ = writer.write_message(Reply::Error(format!("Invalid key Provided")));
                } else if check_format(parts[1]) {
                    let key = parts[1];
                    if let Ok((col, row)) = split_key(key) {
                        let matrix = data_base_matrix.lock().unwrap();

                        if row < matrix.len() && col < matrix[row].len() {
                            let cell_value = &matrix[row][col];
                            match cell_value {
                                CellValue::Error(err) => {
                                    let error_msg = format!("{} = Error: '{}'", key, err);
                                    // let _ = writer.write_message(Reply::Error(error_msg));
                                    print!("{} = ", key);
                                    let _ = writer.write_message(Reply::Error(error_msg));

                                },
                                _ => {
                                    let _ = writer.write_message(Reply::Value(key.to_string(), cell_value.clone()));
                                }
                            };
                            // writer.write_message(reply).ok();
                        }
                    }
                }
            },
            "set" => {
                if parts.len() < 3 {
                    let _ = writer.write_message(Reply::Error(format!("Syntax error")));
                } else if !check_format(parts[1]) {
                    let _ = writer.write_message(Reply::Error(format!("Invalid Key Provided")));
                } else {
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
                        // writer.write_message(Reply::Value(key.to_string(), result)).ok();
                    } else {
                        let result_expression = {
                            let matrix = data_base_matrix.lock().unwrap();
                            replaced_cells_expression(&expression, &matrix)
                        };
                        if let Ok(replaced_expression) = result_expression {
                            // println!("{}", &replaced_expression);
                            if replaced_expression == "None" {
                                // let result = "None".to_string();
                                if let Ok((col, row)) = split_key(key) {
                                    let mut matrix = data_base_matrix.lock().unwrap();
                                    matrix[row][col] = CellValue::None;
                                    // writer.write_message(Reply::Value(key.to_string(), CellValue::None)).ok();
                                }
                            } else {
                                let result = CommandRunner::new(&replaced_expression).run(&HashMap::new());
                                if let Ok((col, row)) = split_key(key) {
                                    let mut matrix = data_base_matrix.lock().unwrap();
                                    set_cell_in_matrix(&mut matrix, col, row, result.clone());
                                    // writer.write_message(Reply::Value(key.to_string(), result)).ok();
                                }
                            }
                            // writer.write_message(Reply::Value(key.to_string(), result)).ok();
                        }
                    }
                }
            },
            _ => writer.write_message(Reply::Error(String::from("Unknown command"))).ok().expect("REASON"),
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


fn replaced_cells_expression(expression: &str, data_base_matrix: &Vec<Vec<CellValue>>) -> Result<String, String> {
    let re = Regex::new(r"([A-Z]+[0-9]+)").unwrap();
    let mut result_expression = String::with_capacity(expression.len());

    let mut last_end = 0;
    for cap in re.captures_iter(expression) {
        let match_str = cap.get(0).unwrap().as_str();
        let (col, row) = split_key(match_str).map_err(|_| "Invalid cell reference")?;
        
        if row < data_base_matrix.len() && col < data_base_matrix[row].len() {
            let value_str = match &data_base_matrix[row][col] {
                CellValue::Int(value) => value.to_string(),
                CellValue::None => "None".to_string(),
                CellValue::Error(_) => "Error".to_string(),
                _ => return Err("Error".to_string()),
            };
            
            result_expression.push_str(&expression[last_end..cap.get(0).unwrap().start()]);
            result_expression.push_str(&value_str);
            last_end = cap.get(0).unwrap().end();
        }
    }

    result_expression.push_str(&expression[last_end..]);
    Ok(result_expression)
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



