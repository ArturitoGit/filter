use crate::args::Column;
use crate::input::Input;

pub fn handle_not_in(source: impl Input, target: impl Input) -> impl Iterator<Item = String> {
    filter_source(source, target,
        |source_field, target_fields| !contains(source_field, target_fields))
}

pub fn handle_also_in(source: impl Input, target: impl Input) -> impl Iterator<Item = String> {
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
            extract(&line, &target_column)
                .map(|it| String::from(it))
        })
        .collect();

    // Filter lines from source on matching fields from target
    source
        .filter(move |line| {
            extract(&line, &source_column)
                .map(|field| filter(field, &target_fields))
                .unwrap_or(false)
        })
}

fn extract<'a>(line: &'a str, column: &Column) -> Option<&'a str> {
    let Column::Column(position, separator) = column else {
        return Some(line);
    };
    if *position <= 0 {
        return Some(line);
    }

    let fields: Vec<&str> = line.split(separator).collect();

    let index = *position - 1; // The Position is 1-based in program arguments
    if fields.len() <= index {
        return None;
    }

    Some(fields[index])
}

fn contains(value: &str, target: &Vec<String>) -> bool {
    target.iter().any(|v| v == value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::IntoIter;
    use crate::args::clone_column;

    struct VecInput {
        lines: IntoIter<String>,
        column: Column
    }

    impl Input for VecInput {
        fn column(&self) -> Column {
            clone_column(&self.column)
        }
    }

    impl Iterator for VecInput {
        type Item = String;
        fn next(&mut self) -> Option<String> {
            self.lines.next()
        }
    }

    fn column(column: usize, separator: &str, lines: Vec<&str>) -> VecInput {
        let lines: Vec<String> = lines.into_iter()
            .map(|it| String::from(it))
            .collect();
        VecInput {
            lines: lines.into_iter(),
            column: Column::Column(column, String::from(separator))
        }
    }

    fn assert_result(actual: impl Iterator<Item = String>, expected: Vec<&str>) {
        let expected_owned: Vec<String> = expected.into_iter()
            .map(|it| String::from(it))
            .collect();
        let result: Vec<String> = actual.collect();
        assert_eq!(expected_owned, result);
    }

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

        assert_result(handle_also_in(s1, s2), vec!["Minerva;Macgonagal", "Hermione;Granger", "Ronald;Weasley"]);
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

        assert_result(handle_not_in(s1, s2), vec!["Harry;Potter"]);
    }
}
