use rsheet_lib::connect::{Manager, Reader, Writer};
use rsheet_lib::replies::Reply;

use std::collections::HashMap;
use std::fmt::Debug;
use regex::Regex;
use std::error::Error;

use log::info;

use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::CommandRunner;
use rsheet_lib::cells::{column_number_to_name, column_name_to_number};

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let (mut recv, mut send) = manager.accept_new_connection().unwrap();
    let mut data_base: HashMap<String, CellValue> = HashMap::new();

    loop {
        info!("Just got message");
        let mut msg = recv.read_message()?;
        // send.write_message(Reply::Error(format!("{msg:?}")))?;
        msg = msg.trim().to_string();
        // println!("{msg:?}");

        let parts: Vec<&str> = msg.split_whitespace().collect();
        match parts.get(0) {
            Some(&"get") => {
                if let Some(value) = parts.get(1).and_then(|&key| data_base.get(key)) {
                    match value {
                        CellValue::Error(err) => {
                            send.write_message(Reply::Error(format!("{} = Error: '{}'", parts[1], err)))
                        },
                        _ => {
                            send.write_message(Reply::Value(parts[1].to_string(), value.clone()))
                        }
                    }?;
                    // println!("{:?}",value.clone());
                } else {
                    send.write_message(Reply::Error(format!("Variable not found")));
                }
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
