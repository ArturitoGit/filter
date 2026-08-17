use crate::args::{Column, Source, CsvFile};

use std::io;
use std::io::{Stdin, Read, Lines, BufReader, BufRead};
use std::fs::File;

#[allow(unused)]
pub trait Input: Iterator<Item = String> {
    fn column(&self) -> Column;
}

pub fn open(source: Source) -> std::io::Result<impl Input> {
    let input = match source {

        Source::File(CsvFile { path, column }) => {
            let file = File::open(path)?;
            StdinOrFile::File(LineReaderInput::from(file), column)
        }

        Source::Stdin(column) => {
            StdinOrFile::Stdin(LineReaderInput::from(io::stdin()), column)
        }
    };
    Ok(input)
}

// Abstract the implementations in an enum
// Wrap them with a column
enum StdinOrFile {
    Stdin(LineReaderInput<Stdin>, Column),
    File(LineReaderInput<File>, Column)
}

impl Input for StdinOrFile {
    fn column(&self) -> Column {
        match self {
            StdinOrFile::Stdin(_, column) => column.clone(),
            StdinOrFile::File(_, column) => column.clone()
        }
    }
}

impl Iterator for StdinOrFile {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        match self {
            StdinOrFile::Stdin(input, _) => input.next(),
            StdinOrFile::File(input, _) => input.next()
        }
    }
}

// Wrap a readable source with a Line-BufReader instance
// And override it to ignre line read errors
struct LineReaderInput<T: Read> {
    lines: Lines<BufReader<T>>
}

impl<T: Read> LineReaderInput<T> {
    fn from(t: T) -> Self {
        Self {
            lines: BufReader::new(t).lines()
        }
    }
}

impl<T: Read> Iterator for LineReaderInput<T> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.lines.next()
            .map(|line| line.unwrap())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::vec::IntoIter;
    use crate::args::{Column};

    pub struct VecInput {
        lines: IntoIter<String>,
        column: Column
    }

    impl Input for VecInput {
        fn column(&self) -> Column {
            self.column.clone()
        }
    }

    impl Iterator for VecInput {
        type Item = String;
        fn next(&mut self) -> Option<String> {
            self.lines.next()
        }
    }

    pub fn column(column: usize, separator: &str, lines: Vec<&str>) -> VecInput {
        let lines: Vec<String> = lines.into_iter()
            .map(|it| String::from(it))
            .collect();
        VecInput {
            lines: lines.into_iter(),
            column: Column::Column(column, String::from(separator))
        }
    }

    pub fn assert_result(actual: impl Iterator<Item = String>, expected: Vec<&str>) {
        let expected_owned: Vec<String> = expected.into_iter()
            .map(|it| String::from(it))
            .collect();
        let result: Vec<String> = actual.collect();
        assert_eq!(expected_owned, result);
    }

}
