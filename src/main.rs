mod parse_args;

use parse_args::parse_args;
use filter::{print_not_in};
use std::env;

fn main() {

    let options = match parse_args(env::args()) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("Invalid argument : {err}");
            return;
        }
    };

    let result = print_not_in(&options);
    if let Err(err) = result {
        eprintln!("Error : {err}");
    }
}
