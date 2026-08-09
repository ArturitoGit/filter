use filter::{Column, Options, Filter};
use filter::Filter::*;
use filter::Column::*;

pub fn parse_args<T: Iterator<Item = String>>(mut args: T) -> Result<Options, String> {

    let mut options_builder = OptionsBuilder::new();

    // Ignore program name
    args.next();

    while let Some(arg) = args.next() {

        if parse_action(&arg, &mut options_builder)? {
            continue;
        }

        if parse_stdin_field_index(&arg, &mut args, &mut options_builder)? {
            continue;
        }

        if parse_stdin_separator(&arg, &mut args, &mut options_builder)? {
            continue;
        }

        // Default : parse last argument as the filename
        parse_arg_file(&arg, &mut options_builder);
    }

    options_builder.build()
}

fn parse_action(next_arg: &str, options: &mut OptionsBuilder) -> Result<bool, String> {
    if next_arg == "--not-in" {
        options.action(NotInArgFile);
        return Ok(true);
    }

    if next_arg == "--also-in" || next_arg == "--in" {
        options.action(AlsoInArgFile);
        return Ok(true);
    }

    return Ok(false);
}


fn parse_stdin_field_index<T>(next_arg: &str, mut args: T, options: &mut OptionsBuilder) -> Result<bool, String>
    where T: Iterator<Item = String>
{

    let mut value: Option<String> = None;

    // -f <index>
    if next_arg == "-f" {

        // Get argument value
        let following_arg = args.next();
        if following_arg.is_none() {
            return Err(String::from("-f must precede a field index"));
        }

        value = following_arg;
    }

    // -f<field>
    else if next_arg.starts_with("-f") {
        value = Some(String::from(&next_arg[2..]));
    }

    // Parse value
    let value = match value {
        None => {
            return Ok(false);
        }
        Some(value) => value
    };

    let field_index: usize = match value.parse() {
        Err(_) => {
            return Err(String::from("the field index argument should be an integer"));
        }
        Ok(size) => size
    };

    options.stdin_field_index(field_index);

    return Ok(false);
}

fn parse_stdin_separator<T>(next_arg: &str, mut args: T, options: &mut OptionsBuilder) -> Result<bool, String>
    where T: Iterator<Item = String>
{

    // -s <separator>
    if next_arg == "-s" {

        // Get argument value
        let separator = match args.next() {
            None => {
                return Err(String::from("-s must precede a separator"));
            }
            Some(value) => value
        };

        // Update options
        options.stdin_separator(String::from(separator));

        return Ok(true);
    }

    // -s<field>
    if next_arg.starts_with("-s") {
        let separator = &next_arg[2..];
        options.stdin_separator(String::from(separator));
    }

    return Ok(false);
}


fn parse_arg_file(arg: &str, options_builder: &mut OptionsBuilder) {
    let segments: Vec<&str> = arg.split(":").collect();

    let filename = segments[0];
    options_builder.arg_file_name(String::from(filename));
    if segments.len() == 1 {
        return;
    }

    let mut index: usize = 0;
    if segments.len() >= 2 {
        index = segments[1].parse().unwrap();
    }

    let mut separator = String::from(";");
    if segments.len() >= 3 {
        separator = String::from(segments[2]);
    }

    let column = ColumnInfo(index, separator);
    options_builder.arg_file_column(column);
}

struct OptionsBuilder {
    action: Filter,
    arg_file_name: Option<String>,
    arg_file_column: Column,
    stdin_column: Column
}

impl OptionsBuilder {
    fn new() -> Self {
        Self {
            action: NotInArgFile,
            arg_file_name: None,
            arg_file_column: EntireLine,
            stdin_column: EntireLine
        }
    }

    fn build(self) -> Result<Options, String> {

        let arg_file_name = match self.arg_file_name {
            None => {
                return Err(String::from("Missing filename"));
            }
            Some(name) => name
        };

        Ok(Options {
            action: self.action,
            stdin_column: self.stdin_column,
            arg_file_name,
            arg_file_column: self.arg_file_column
        })
    }

    fn arg_file_name(&mut self, name: String) {
        self.arg_file_name = Some(name);
    }

    fn arg_file_column(&mut self, column: Column) {
        self.arg_file_column = column;
    }

    fn stdin_separator(&mut self, separator: String) {
        let field_index = match self.stdin_column {
            EntireLine => 0,
            ColumnInfo(index, _) => index
        };
        self.stdin_column = ColumnInfo(field_index, separator);
    }

    fn stdin_field_index(&mut self, field_index: usize) {
        let separator = match &self.stdin_column {
            EntireLine => String::from(";"),
            ColumnInfo(_, separator) => String::from(separator)
        };
        self.stdin_column = ColumnInfo(field_index, separator);
    }

    fn action(&mut self, action: Filter) {
        self.action = action;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(cmd_line: &str) -> Result<Options, String> {
        let args = cmd_line.split(' ').map(|s| String::from(s));
        parse_args(args)
    }

    fn assert_ok_options(actual: &Result<Options, String>, expected: Options) {
        match actual {
            Err(err) => {
                panic!("{err}");
            }
            Ok(actual) => {
                assert_eq!(&expected, actual);
            }
        }
    }

    fn assert_err(actual: &Result<Options, String>, expected_err: &str) {
        match actual {
            Ok(_) => {
                panic!("Expected error");
            }
            Err(err) => {
                assert_eq!(expected_err, err.to_string());
            }
        }
    }

    #[test]
    fn single_filename() {
        let result = parse("filter --not-in test.csv");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.csv"),
            arg_file_column: EntireLine,
            stdin_column: EntireLine
        });
    }

    #[test]
    fn filename_field_index() {
        let result = parse("filter --not-in test.csv:3");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.csv"),
            arg_file_column: ColumnInfo(3, String::from(";")),
            stdin_column: EntireLine
        });
    }

    #[test]
    fn filename_arg_file_column() {
        let result = parse("filter --not-in test.tsv:3:,");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: ColumnInfo(3, String::from(",")),
            stdin_column: EntireLine
        });
    }

    #[test]
    fn missing_filename() {
        let result = parse("filter --not-in");
        assert_err(&result, "Missing filename");
    }

    #[test]
    fn stdin_field_index() {
        let result = parse("filter --not-in -f 7 test.tsv:3:,");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: ColumnInfo(3, String::from(",")),
            stdin_column: ColumnInfo(7, String::from(";"))
        });
    }

    #[test]
    fn invalid_field_index() {
        let result = parse("filter --not-in -f some test.tsv:3:,");
        assert_err(&result, "the field index argument should be an integer");
    }

    #[test]
    fn stdin_separator() {
        let result = parse("filter --not-in -s | test.tsv:3:,");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: ColumnInfo(3, String::from(",")),
            stdin_column: ColumnInfo(0, String::from("|"))
        });
    }

    #[test]
    fn stdin_field_index_onearg() {
        let result = parse("filter --not-in -f10 test.tsv");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: EntireLine,
            stdin_column: ColumnInfo(10, String::from(";"))
        });
    }

    #[test]
    fn stdin_separator_close() {
        let result = parse("filter --not-in -s| test.tsv");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: EntireLine,
            stdin_column: ColumnInfo(0, String::from("|"))
        });
    }

    #[test]
    fn complete() {
        let result = parse("filter --not-in -f 7 -s : test.tsv:2:+");
        assert_ok_options(&result, Options {
            action: NotInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: ColumnInfo(2, String::from("+")),
            stdin_column: ColumnInfo(7, String::from(":"))
        });
    }

    #[test]
    fn action_also_in() {
        let result = parse("filter -f 7 -s : --also-in test.tsv:2:+");
        assert_ok_options(&result, Options {
            action: AlsoInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: ColumnInfo(2, String::from("+")),
            stdin_column: ColumnInfo(7, String::from(":"))
        });
    }

    #[test]
    fn action_also_in_short() {
        let result = parse("filter --in test.tsv");
        assert_ok_options(&result, Options {
            action: AlsoInArgFile,
            arg_file_name: String::from("test.tsv"),
            arg_file_column: EntireLine,
            stdin_column: EntireLine
        });
    }
}
