use std::error::Error;
use std::collections::HashMap;

use crate::args::Arguments;
use crate::input::{open, Input};

pub fn handle_duplicates(args: Arguments) -> Result<(), Box<dyn Error>> {

    let Arguments { source, .. } = args;

    let source = open(source)?;
    for output in filter_duplicates(source) {
        println!("{output}");
    }

    Ok(())
}

fn filter_duplicates(source: impl Input) -> impl Iterator<Item = String> {
    let column = source.column();
    let mut stats: HashMap<String, usize> = HashMap::new();

    let lines: Vec<String> = source
        .filter_map(|line| {
            let field = column.extract(&line)?;
            stats.entry(field.to_string())
                .and_modify(|occurences| *occurences += 1)
                .or_insert(1);

            Some(line)
        })
        .collect();

    lines.into_iter()
        .filter(move |line| {
            let field = column.extract(&line).unwrap();
            let occurences = stats.get(field)
                .map(|it| *it)
                .unwrap_or(0);
            occurences > 1
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::tests::*;

    #[test]
    fn it_works() {
        let source = column(2, ";", vec![
            "James;Potter",
            "Minerva;Macgonagal",
            "Harry;Potter",
            "Hermione;Granger",
            "Lilly;Potter"
        ]);

        assert_result(filter_duplicates(source), vec!["James;Potter", "Harry;Potter", "Lilly;Potter"]);
    }

    #[test]
    fn it_keeps_input_order() {
        let source = column(2, ";", vec![
            "James;Potter",
            "Ron;Weasley",
            "Arthur;Weasley",
            "Harry;Potter",
            "Hermione;Granger"
        ]);

        assert_result(filter_duplicates(source), vec!["James;Potter", "Ron;Weasley", "Arthur;Weasley", "Harry;Potter"]);
    }
}
