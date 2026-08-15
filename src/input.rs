use crate::args::{Column, Source, CsvFile, clone_column};

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
            StdinOrFile::Stdin(_, column) => clone_column(&column),
            StdinOrFile::File(_, column) => clone_column(&column)
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
