mod arg;
mod handlers;

use arg::parse_args::parse;
use arg::args::FilterType::*;
use handlers::{also_in, duplicates};

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse(env::args())?;
    match &args.filter {
        NotIn | AlsoIn =>  also_in::handle(args),
        Duplicates | Uniques => duplicates::handle(args)
    }
}
