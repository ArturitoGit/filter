use std::error::Error;
use std::collections::HashMap;

use crate::args::Arguments;
use crate::args::FilterType::*;
use crate::input::{open, Input};

pub fn handle_duplicates(args: Arguments) -> Result<(), Box<dyn Error>> {

    let Arguments { filter, source, .. } = args;

    let source = open(source)?;

    match filter {
        Duplicates => print_all(filter_duplicates(source)),
        _ =>          print_all(filter_uniques(source))
    }

    Ok(())
}

fn print_all(iter: impl Iterator<Item = String>) {
    for line in iter {
        println!("{line}");
    }
}

fn filter_duplicates(source: impl Input) -> impl Iterator<Item = String> {
    filter_by_occurence(source, |occurences| occurences > 1)
}

fn filter_uniques(source: impl Input) -> impl Iterator<Item = String> {
    filter_by_occurence(source, |occurences| occurences == 1)
}

fn filter_by_occurence<F>(source: impl Input, predicate: F) -> impl Iterator<Item = String>
where F: Fn(usize) -> bool
{
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
            predicate(occurences)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::tests::*;

    #[test]
    fn test_filter_duplicates() {
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
    fn test_filter_uniques() {
        let source = column(2, ";", vec![
            "James;Potter",
            "Minerva;Macgonagal",
            "Harry;Potter",
            "Hermione;Granger",
            "Lilly;Potter"
        ]);

        assert_result(filter_uniques(source), vec!["Minerva;Macgonagal", "Hermione;Granger"]);
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
