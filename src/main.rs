mod args;
mod parse_args;
mod input;
mod dispatch;
mod handle_compare;

use parse_args::parse;
use dispatch::handle;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse(env::args())?;
    handle(args)?;
    Ok(())
}
