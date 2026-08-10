use std::io;
use std::io::{BufReader,BufRead};
use std::fs::File;
use crate::Filter::*;
use crate::Column::*;

#[derive(PartialEq, Debug)]
pub enum Filter {
    NotInArgFile,
    AlsoInArgFile
}

#[derive(PartialEq, Debug)]
pub struct Options {
    pub action: Filter,
    pub stdin_column: Column,
    pub arg_file_name: String,
    pub arg_file_column: Column
}

#[derive(PartialEq, Debug)]
pub enum Column {
    EntireLine,
    ColumnInfo(usize, String)
}

pub fn handle(options: &Options) -> std::io::Result<()> {
    match &options.action {
        NotInArgFile => {
            print_not_in(options)
        }
        AlsoInArgFile => {
            print_also_in(options)
        }
    }
}

// Read lines from stdin, print those who are not present in arg file
fn print_not_in(options: &Options) -> std::io::Result<()> {

    let arg_file_fields = parse_arg_file_fields(&options)?;

    parse_stdin_fields(options)
        .filter(|(_, field)| !contains(&field, &arg_file_fields))
        .for_each(|(line, _)| { println!("{}", &line); });

    Ok(())
}

// Read lines from stdin, print those who are also present in arg file
fn print_also_in(options: &Options) -> std::io::Result<()> {

    let arg_file_fields = parse_arg_file_fields(&options)?;

    parse_stdin_fields(options)
        .filter(|(_, field)| contains(&field, &arg_file_fields))
        .for_each(|(line, _)| { println!("{}", &line); });

    Ok(())
}

// Parse given column in given file
fn parse_arg_file_fields(options: &Options) -> std::io::Result<Vec<String>> {
    let mut values = Vec::new();

    let reader = BufReader::new(File::open(&options.arg_file_name)?);
    for line in reader.lines() {
        let line = line?;
        let field = extract(&options.arg_file_column, &line).unwrap_or("");
        values.push(String::from(field));
    }

    Ok(values)
}

// Extract the field from given column in given line
fn extract<'a>(column: &Column, line: &'a str) -> Option<&'a str> {
    return match column {
        EntireLine => Some(line),

        ColumnInfo(field, separator) => {
            if *field == 0 {
                return Some(line);
            }
            let index = *field - 1; // 0-based in code, 1-based in arguments
            let fields: Vec<&str> = line.split(separator).collect();
            if fields.len() <= index {
                return None;
            }
            Some(fields[index])
        }
    }
}

fn parse_stdin_fields(options: &Options) -> impl Iterator<Item = (String, String)> {
    io::stdin().lines()
        .map(|line| line.expect("Failed to read line from stdin"))
        .map(|line| {
            match extract(&options.stdin_column, &line) {
                Some(field) => Some((String::from(&line), String::from(field))),
                None => {
                    eprintln!("Could not parse field from line {}", &line);
                    None
                }
            }
        })
        .filter(|field| field.is_some())
        .map(|field| field.unwrap())
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
        let result = extract_result(3, ";", "Hello;je;suis;Arthur");
        assert_some("suis", result);
    }

    #[test]
    fn it_returns_none_on_exceeding_index() {
        let result = extract_result(5, ";", "Hello;je;suis;Arthur");
        assert_none(result);
    }

    #[test]
    fn it_handles_different_separators() {
        let result = extract_result(1, "|", "Hello|je|suis|Arthur");
        assert_some("Hello", result);
    }

    #[test]
    fn it_handles_0_as_entire_line() {
        let result = extract_result(0, "|", "Hello|je|suis|Arthur");
        assert_some("Hello|je|suis|Arthur", result);
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
