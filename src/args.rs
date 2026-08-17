#[derive(PartialEq, Debug)]
pub struct Arguments {
    pub source: Source,
    pub filter: FilterType,
    pub target: Option<CsvFile>,
}

#[derive(PartialEq, Debug)]
pub enum FilterType {
    NotIn,
    AlsoIn,
    Duplicates,
    Uniques
}

#[derive(PartialEq, Debug)]
pub enum Source {
    File(CsvFile),
    Stdin(Column)
}

#[derive(PartialEq, Debug)]
pub struct CsvFile {
    pub path: String,
    pub column: Column,
}

#[derive(PartialEq, Debug)]
pub enum Column {
    EntireLine,
    Column(usize, String)
}

impl Column {
    pub fn clone(&self) -> Column {
        match &self {
            Column::EntireLine => Column::EntireLine,
            Column::Column(size, sep) => Column::Column(*size, String::from(sep))
        }
    }

    pub fn extract<'a>(&self, line: &'a str) -> Option<&'a str> {
        let Column::Column(position, separator) = self else {
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
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn extract_entire_line() {
        let line = "Je m'appelle Henry";
        let parsed = Column::EntireLine.extract(line);
        assert_eq!("Je m'appelle Henry", parsed.unwrap());
    }

    #[test]
    fn extract_is_1_based() {
        let line = "Henry;4;petit";
        let column = Column::Column(2, String::from(";"));
        assert_eq!("4", column.extract(line).unwrap());
    }

    #[test]
    fn extract_none_on_not_enough_fields() {
        let line = "Henry;4;petit";
        let column = Column::Column(7, String::from(";"));
        assert_eq!(true, column.extract(line).is_none());
    }

    #[test]
    fn extract_entire_line_on_column_0() {
        let line = "Henry;4;petit";
        let column = Column::Column(0, String::from(";"));
        assert_eq!("Henry;4;petit", column.extract(line).unwrap());
    }
}
