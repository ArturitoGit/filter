mod args;
mod parse_args;
mod input;
mod handle_compare;
mod duplicates;

use parse_args::parse;
use args::FilterType::*;
use handle_compare::{handle_not_in, handle_also_in};
use duplicates::handle_duplicates;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse(env::args())?;
    match &args.filter {
        NotIn =>  handle_not_in(args),
        AlsoIn => handle_also_in(args),
        Duplicates => handle_duplicates(args)
    }
}
