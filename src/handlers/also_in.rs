use crate::arg::args::{
    Arguments, Source, FilterType,
    FilterType::*
};
use crate::handlers::input::{open, Input};

use std::error::Error;

pub fn handle(args: Arguments) -> Result<(), Box<dyn Error>> {
    let Arguments { source, target, filter } = args;

    // The target file is mandatory
    let Some(target) = target else {
        return Err(error("A target file is required for this filter"));
    };

    let source = open(source)?;
    let target = open(Source::File(target))?;

    filter_source(source, filter, target)
        .for_each(|line| println!("{line}"));

    Ok(())
}

fn filter_source(source: impl Input, filter: FilterType, target: impl Input) -> impl Iterator<Item = String>
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
            .map(|field| check(field, &filter, &target_fields))
            .unwrap_or(false)
    })
}

fn check(field: &str, filter: &FilterType, target_field: &Vec<String>) -> bool {
    match filter {
        NotIn => !target_field.iter().any(|v| v == field),
        AlsoIn => target_field.iter().any(|v| v == field),
        _ => panic!("Invalid filter type")
    }
}

fn error(msg: &str) -> Box<dyn Error> {
    Box::<dyn Error>::from(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::input::tests::*;

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

        assert_result(filter_source(s1, AlsoIn, s2), vec!["Minerva;Macgonagal", "Hermione;Granger", "Ronald;Weasley"]);
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

        assert_result(filter_source(s1, NotIn, s2), vec!["Harry;Potter"]);
    }
}
