mod parse_args;

use parse_args::parse_args;
use filter::handle;
use std::env;

fn main() {

    let options = match parse_args(env::args()) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("Invalid argument : {err}");
            return;
        }
    };

    let result = handle(&options);
    if let Err(err) = result {
        eprintln!("Error : {err}");
    }
}
