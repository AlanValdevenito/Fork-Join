extern crate serde;
extern crate serde_json;

use std::env;

mod output;
use output::Output;

mod player;
use player::Player;

mod weapon;
use weapon::Weapon;

mod utils;
use utils::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = match args.get(1) {
        Some(arg) => {
            if let Ok(path) = arg.parse::<String>() {
                path
            } else {
                eprintln!("Error: Argument is not a valid path");
                return;
            }
        }

        None => {
            eprintln!("Error: You must provide a path as an argument");
            return;
        }
    };

    let num_threads = match args.get(2) {
        Some(arg) => {
            if let Ok(num_threads) = arg.parse::<usize>() {
                num_threads
            } else {
                eprintln!("Error: Argument is not a valid number of threads");
                return;
            }
        }

        None => {
            eprintln!("Error: You must provide a number of threads as an argument");
            return;
        }
    };

    let output_file_name = match args.get(3) {
        Some(arg) => {
            if let Ok(output_file_name) = arg.parse::<String>() {
                output_file_name
            } else {
                eprintln!("Error: Argument is not a valid file name");
                return;
            }
        }

        None => {
            eprintln!("Error: You must provide a file name as an argument");
            return;
        }
    };

    let output: Output = process_files(path, num_threads);

    if let Err(e) = output.write_to_file(&output_file_name) {
        eprintln!("Error: {}", e);
    }
}
