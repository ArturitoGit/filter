use crate::args::{Arguments, FilterType, CsvFile, Source, Column};
use crate::args::FilterType::*;
use crate::args::Column::*;

pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Arguments, String> {

    // Ignore the program name
    args.next();

    let mut builder = ArgsBuilder::new();

    while let Some(arg) = args.next() {
        parse_arg(&arg, &mut args, &mut builder)?;
    }

    builder.build()
}

struct ArgsBuilder {
    opts: Opts,
    source: Option<RawCsvFile>,
    filter: Option<FilterType>,
    target: Option<RawCsvFile>
}

#[derive(PartialEq, Debug)]
pub struct Opts {
    pub column_idx: usize,
    pub separator: String
}

#[derive(PartialEq, Debug)]
pub struct RawCsvFile {
    pub path: String,
    pub column_idx: Option<usize>,
    pub separator: Option<String>
}

impl ArgsBuilder {
    fn new() -> Self {
        Self {
            opts: Opts {
                column_idx: 0,
                separator: String::from(";")
            },
            source: None,
            filter: None,
            target: None,
        }
    }
    fn build(self) -> Result<Arguments, String> {
        // Filter is mandatory
        let Some(filter) = self.filter else {
            return Err(String::from("At least one filter should be specified"));
        };

        let target = self.target.map(|f| build_file(f, &self.opts));
        let source = self.source.map(|f| build_file(f, &self.opts))
            .map(|file| Source::File(file))
            .unwrap_or(stdin_source(&self.opts));

        Ok(Arguments { source, filter, target })
    }
    fn set_file(&mut self, file: RawCsvFile) {
        match self.filter {
            None => {
                self.source = Some(file);
            },
            Some(_) => {
                self.target = Some(file);
            }
        }
    }
}

fn build_file(file: RawCsvFile, opts: &Opts) -> CsvFile {
    CsvFile {
        path: file.path,
        column: build_column(
            file.column_idx.unwrap_or(opts.column_idx),
            file.separator.as_ref().unwrap_or(&opts.separator)
        )
    }
}

fn stdin_source(opts: &Opts) -> Source {
    Source::Stdin(build_column(opts.column_idx, &opts.separator))
}

fn build_column(col: usize, sep: &str) -> Column {
    if col == 0 {
        return EntireLine;
    }
    Column(col, String::from(sep))
}

fn parse_arg(arg: &str, args: &mut impl Iterator<Item = String>, builder: &mut ArgsBuilder) -> Result<(), String>{

    if let Some(column_idx) = try_parse_opts_column(arg, args)? {
        builder.opts.column_idx = column_idx;
        return Ok(());
    }

    if let Some(separator) = try_parse_opts_separator(arg, args)? {
        builder.opts.separator = separator;
        return Ok(());
    }

    if let Some(filter) = try_parse_filter(arg)? {
        builder.filter = Some(filter);
        return Ok(());
    }

    // If not anything else, the argument must be a file reference
    builder.set_file(parse_csv_file(arg)?);
    Ok(())
}

fn try_parse_filter(arg: &str) -> Result<Option<FilterType>, String> {
    let filter = match arg {
        "--not-in" => NotIn,
        "--also-in" => AlsoIn,
        "--in" => AlsoIn,
        "--duplicates" => Duplicates,
        _ => {
            return Ok(None);
        }
    };
    Ok(Some(filter))
}

fn try_parse_opts_column(arg: &str, args: &mut impl Iterator<Item = String>) -> Result<Option<usize>, String> {
    let value = match arg {
        "-f" => {
            let Some(following_arg) = args.next() else {
                return Err(String::from("-f must precede a field index"));
            };
            following_arg
        },

        _ if arg.starts_with("-f") => {
            String::from(&arg[2..])
        },

        _ => {
            return Ok(None);
        }
    };

    let Ok(column_idx) = value.parse() else {
        return Err(format!("Invalid column index : {value}"));
    };

    Ok(Some(column_idx))
}

fn try_parse_opts_separator(arg: &str, args: &mut impl Iterator<Item = String>) -> Result<Option<String>, String> {
    // Get value
    let separator = match arg {
        "-s" => {
            let Some(following_arg) = args.next() else {
                return Err(String::from("-f must precede a field index"));
            };
            following_arg
        },

        _ if arg.starts_with("-s") => {
            String::from(&arg[2..])
        },

        _ => {
            return Ok(None);
        }
    };

    Ok(Some(separator))
}

fn parse_csv_file(arg: &str) -> Result<RawCsvFile, String> {
    let segments: Vec<&str> = arg.split(':').collect();
    Ok(RawCsvFile {
        path: segments[0].to_string(),
        column_idx: match segments.get(1) {
            None => None,
            Some(&"") => None,
            Some(segment) => {
                let Ok(column_idx) = segment.parse() else {
                    return Err(format!("Invalid column index : {segment}"));
                };
                Some(column_idx)
            }
        },
        separator: segments.get(2)
            .filter(|it| !it.is_empty())
            .map(|it| it.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parse(cmd_line: &str, expected: Arguments) {
        let args = cmd_line.split(' ').map(|s| String::from(s));
        match parse(args) {
            Ok(result) => {
                assert_eq!(result, expected);
            },
            Err(err) => {
                panic!("{err}");
            }
        }
    }

    fn assert_err(cmd_line: &str, expected_err: &str) {
        let args = cmd_line.split(' ').map(|s| String::from(s));
        match parse(args) {
            Ok(_) => {
                panic!("Expected error, got none");
            },
            Err(err) => {
                assert_eq!(err, expected_err);
            }
        }
    }

    #[test]
    fn it_works() {
        assert_parse("filter some.csv --not-in other.tsv", Arguments {
            source: Source::File(CsvFile {
                path: String::from("some.csv"),
                column: EntireLine,
            }),
            filter: NotIn,
            target: Some(CsvFile {
                path: String::from("other.tsv"),
                column: EntireLine,
            })
        });
    }

    #[test]
    fn parses_files_with_columns() {
        assert_parse("filter some.csv:2 --not-in other.tsv:3", Arguments {
            source: Source::File(CsvFile {
                path: String::from("some.csv"),
                column: Column(2, String::from(";"))
            }),
            filter: NotIn,
            target: Some(CsvFile {
                path: String::from("other.tsv"),
                column: Column(3, String::from(";"))
            })
        });
    }

    #[test]
    fn parses_files_with_separator() {
        assert_parse("filter some.csv:1:| --not-in other.tsv:4:-", Arguments {
            source: Source::File(CsvFile {
                path: String::from("some.csv"),
                column: Column(1, String::from("|"))
            }),
            filter: NotIn,
            target: Some(CsvFile {
                path: String::from("other.tsv"),
                column: Column(4, String::from("-"))
            })
        });
    }

    #[test]
    fn parses_no_file() {
        assert_parse("filter --duplicates", Arguments {
            source: Source::Stdin(EntireLine),
            filter: Duplicates,
            target: None
        });
    }

    #[test]
    fn parses_options() {
        assert_parse("filter -f32 -s+ --in other.txt", Arguments {
            source: Source::Stdin(Column(32, String::from("+"))),
            filter: AlsoIn,
            target: Some(CsvFile {
                path: String::from("other.txt"),
                column: Column(32, String::from("+"))
            })
        });
    }

    #[test]
    fn parses_override_option() {
        assert_parse("filter -f5 -s+ --in other.txt:3", Arguments {
            source: Source::Stdin(Column(5, String::from("+"))),
            filter: AlsoIn,
            target: Some(CsvFile {
                path: String::from("other.txt"),
                column: Column(3, String::from("+"))
            })
        });
    }

    #[test]
    fn parses_column_option() {
        assert_parse("filter -f32 --in other.txt", Arguments {
            source: Source::Stdin(Column(32, String::from(";"))),
            filter: AlsoIn,
            target: Some(CsvFile {
                path: String::from("other.txt"),
                column: Column(32, String::from(";"))
            })
        });
    }

    #[test]
    fn parses_source() {
        assert_parse("filter some.txt --duplicates", Arguments {
            source: Source::File(CsvFile {
                path: String::from("some.txt"),
                column: EntireLine
            }),
            filter: Duplicates,
            target: None
        });
    }

    #[test]
    fn parses_target() {
        assert_parse("filter --also-in other.tsv", Arguments {
            source: Source::Stdin(EntireLine),
            filter: AlsoIn,
            target: Some(CsvFile {
                path: String::from("other.tsv"),
                column: EntireLine
            })
        });
    }

    #[test]
    fn ignore_file_empty_field() {
        assert_parse("filter --also-in other.tsv:", Arguments {
            source: Source::Stdin(EntireLine),
            filter: AlsoIn,
            target: Some(CsvFile {
                path: String::from("other.tsv"),
                column: EntireLine
            })
        });
    }

    #[test]
    fn ignore_file_empty_separator() {
        assert_parse("filter --also-in other.tsv:3:", Arguments {
            source: Source::Stdin(EntireLine),
            filter: AlsoIn,
            target: Some(CsvFile {
                path: String::from("other.tsv"),
                column: Column(3, String::from(";"))
            })
        });
    }

    #[test]
    fn fails_on_no_filter() {
        assert_err("filter -f2 some.txt other.tsv", "At least one filter should be specified");
    }

    #[test]
    fn fails_on_file_invalid_field() {
        assert_err("filter --not-in other.tsv:u", "Invalid column index : u");
    }

    #[test]
    fn fails_on_opts_invalid_field() {
        assert_err("filter -ftest --not-in other.tsv", "Invalid column index : test");
    }

    #[test]
    fn fails_on_opts_missing_field() {
        assert_err("filter --not-in other.tsv -f", "-f must precede a field index");
    }
}
