use crate::args::{Arguments, Source};
use crate::args::FilterType::*;
use crate::input::open;
use crate::not_in;

use std::error::Error;

pub fn handle(args: Arguments) -> Result<(), Box<dyn Error>> {
    match args.filter {
        NotIn => {
            handle_not_in(args)
        }
        _ => {
            handle_not_in(args)
        }
    }
}

fn handle_not_in(args: Arguments) -> Result<(), Box<dyn Error>> {

    let Arguments { source, target, .. } = args;

    // The target file is mandatory
    let Some(target) = target else {
        return error("A target file is required for the --not-in filter");
    };

    // Open source & target
    let source_input = open(source)?;
    let target_input = open(Source::File(target))?;

    // Handle sources and print result to stdout
    for output in not_in::handle(source_input, target_input) {
        println!("{output}");
    }

    Ok(())
}

fn error(msg: &str) -> Result<(), Box<dyn Error>> {
    Err(Box::<dyn Error>::from(msg))
}
