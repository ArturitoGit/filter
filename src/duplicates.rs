use std::error::Error;
use std::collections::HashMap;

use crate::args::{Arguments, FilterType};
use crate::args::FilterType::*;
use crate::input::{open, Input};

pub fn handle_duplicates(args: Arguments) -> Result<(), Box<dyn Error>> {

    let Arguments { source, filter, .. } = args;

    let source = open(source)?;

    filter_by(source, filter)
        .for_each(|line| println!("{line}"));

    Ok(())
}

fn filter_by(source: impl Input, filter: FilterType) -> impl Iterator<Item = String>
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

            check(occurences, &filter)
        })
}

fn check(occurences: usize, filter: &FilterType) -> bool {
    match filter {
        Uniques => occurences == 1,
        Duplicates => occurences > 1,
        _ => panic!("Invalid filter type")
    }
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

        assert_result(filter_by(source, Duplicates), vec!["James;Potter", "Harry;Potter", "Lilly;Potter"]);
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

        assert_result(filter_by(source, Uniques), vec!["Minerva;Macgonagal", "Hermione;Granger"]);
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

        assert_result(filter_by(source, Duplicates), vec!["James;Potter", "Ron;Weasley", "Arthur;Weasley", "Harry;Potter"]);
    }
}
