use std::io;
use std::io::{BufReader,BufRead};
use std::fs::File;

pub struct Options {
    pub stdin_column: Column,
    pub arg_file_column: Column,
    pub arg_file_name: String
}

pub enum Column {
    EntireLine,
    ColumnInfo(usize, String)
}

// Read lines from stdin, print those who are not present in arg file
pub fn print_not_in(options: &Options) -> std::io::Result<()> {

    let arg_file_fields = parse(&options.arg_file_column, &options.arg_file_name)?;

    for line in io::stdin().lines() {
        let line = line?;
        match extract(&options.stdin_column, &line) {
            None => {
                eprintln!("Could not parse field from line {}", &line);
            }
            Some(field) => {
                if !contains(field, &arg_file_fields) {
                    println!("{}", &line);
                }
            }
        }
    }

    Ok(())
}

// Parse given column in given file
fn parse(column: &Column, filename: &str) -> std::io::Result<Vec<String>> {
    let mut values = Vec::new();

    let reader = BufReader::new(File::open(filename)?);
    for line in reader.lines() {
        let line = line?;
        let field = extract(&column, &line).unwrap_or("");
        values.push(String::from(field));
    }

    Ok(values)
}

// Extract the field from given column in given line
fn extract<'a>(column: &Column, line: &'a str) -> Option<&'a str> {
    return match column {
        Column::EntireLine => Some(line),

        Column::ColumnInfo(index, separator) => {
            let fields: Vec<&str> = line.split(separator).collect();
            if fields.len() <= *index {
                return None;
            }
            Some(fields[*index])
        }
    }
}

fn contains(value: &str, vec: &Vec<String>) -> bool {
    for item in vec {
        if value == &item[..] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_result<'a>(index: usize, separator: &str, line: &'a str) -> Option<&'a str> {
        let column = Column::ColumnInfo(index, String::from(separator));
        return extract(&column, &line);
    }

    fn assert_some<T>(expected: T, value: Option<T>)
        where T: PartialEq + std::fmt::Debug
    {
        assert_eq!(true, value.is_some(), "Should be some");
        assert_eq!(expected, value.unwrap());
    }

    fn assert_none<T>(value: Option<T>) {
        assert_eq!(true, value.is_none(), "Should be none");
    }

    #[test]
    fn it_parses_a_column() {
        let result = extract_result(2, ";", "Hello;je;suis;Arthur");
        assert_some("suis", result);
    }

    #[test]
    fn it_returns_none_on_exceeding_index() {
        let result = extract_result(10, ";", "Hello;je;suis;Arthur");
        assert_none(result);
    }

    #[test]
    fn it_handles_different_separators() {
        let result = extract_result(1, "|", "Hello|je|suis|Arthur");
        assert_some("je", result);
    }

    #[test]
    fn test_contains_returns_true() {
        assert_eq!(true, contains("some", &vec![String::from("one"), String::from("some")]));
    }

    #[test]
    fn test_contains_returns_false() {
        assert_eq!(false, contains("some", &vec![]));
    }
}
