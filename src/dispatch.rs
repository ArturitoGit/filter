use crate::args::{Arguments, Source};
use crate::args::FilterType::*;
use crate::input::{open, Input};
use crate::handle_compare;

use std::error::Error;

pub fn handle(args: Arguments) -> Result<(), Box<dyn Error>> {
    match args.filter {
        NotIn =>  handle_not_in(args),
        AlsoIn => handle_also_in(args),
        _ => {
            handle_not_in(args)
        }
    }
}

fn handle_not_in(args: Arguments) -> Result<(), Box<dyn Error>> {

    let (source, target) = open_sources(args)?;

    for output in handle_compare::handle_not_in(source, target) {
        println!("{output}");
    }

    Ok(())
}

fn handle_also_in(args: Arguments) -> Result<(), Box<dyn Error>> {

    let (source, target) = open_sources(args)?;

    for output in handle_compare::handle_also_in(source, target) {
        println!("{output}");
    }

    Ok(())
}

fn open_sources(args: Arguments) -> Result<(impl Input, impl Input), Box<dyn Error>> {
    let Arguments { source, target, .. } = args;

    // The target file is mandatory
    let Some(target) = target else {
        return Err(error("A target file is required for this filter"));
    };

    let source_input = open(source)?;
    let target_input = open(Source::File(target))?;

    Ok((source_input, target_input))
}

fn error(msg: &str) -> Box<dyn Error> {
    Box::<dyn Error>::from(msg)
}
