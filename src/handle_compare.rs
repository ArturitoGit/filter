use crate::args::{Arguments, Source};
use crate::input::{open, Input};

use std::error::Error;

pub fn handle_not_in(args: Arguments) -> Result<(), Box<dyn Error>> {
    let (source, target) = open_sources(args)?;
    for output in filter_not_in(source, target) {
        println!("{output}");
    }
    Ok(())
}

pub fn handle_also_in(args: Arguments) -> Result<(), Box<dyn Error>> {
    let (source, target) = open_sources(args)?;
    for output in filter_also_in(source, target) {
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

fn filter_not_in(source: impl Input, target: impl Input) -> impl Iterator<Item = String> {
    filter_source(source, target,
        |source_field, target_fields| !contains(source_field, target_fields))
}

fn filter_also_in(source: impl Input, target: impl Input) -> impl Iterator<Item = String> {
    filter_source(source, target,
        |source_field, target_fields| contains(source_field, target_fields))
}

fn filter_source<F>(source: impl Input, target: impl Input, filter: F) -> impl Iterator<Item = String>
where F: Fn(&str, &Vec<String>) -> bool
{
    let source_column = source.column();
    let target_column = target.column();

    // Collect fields from target
    let target_fields: Vec<String> = target
        .filter_map(|line| {
            target_column.extract(&line)
                .map(|it| String::from(it))
        })
        .collect();

    // Filter lines from source on matching fields from target
    source.filter(move |line| {
        source_column.extract(&line)
            .map(|field| filter(field, &target_fields))
            .unwrap_or(false)
    })
}

fn contains(value: &str, target: &Vec<String>) -> bool {
    target.iter().any(|v| v == value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::tests::*;

    #[test]
    fn it_handles_fields() {
        let s1 = column(2, ";", vec![
            "Minerva;Macgonagal",
            "Harry;Potter",
            "Hermione;Granger",
            "Ronald;Weasley"
        ]);

        let s2 = column(1, ",", vec![
            "Granger,1",
            "Macgonagal,2",
            "Weasley,3"
        ]);

        assert_result(filter_also_in(s1, s2), vec!["Minerva;Macgonagal", "Hermione;Granger", "Ronald;Weasley"]);
    }

    #[test]
    fn test_also_in() {
        let s1 = column(2, ";", vec![
            "Minerva;Macgonagal",
            "Harry;Potter",
            "Hermione;Granger",
            "Ronald;Weasley"
        ]);

        let s2 = column(1, ",", vec![
            "Granger,1",
            "Macgonagal,2",
            "Weasley,3"
        ]);

        assert_result(filter_not_in(s1, s2), vec!["Harry;Potter"]);
    }
}
