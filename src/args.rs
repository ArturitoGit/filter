#[derive(PartialEq, Debug)]
pub struct Arguments {
    pub source: Source,
    pub filter: FilterType,
    pub target: Option<CsvFile>,
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

pub fn clone_column(col: &Column) -> Column {
    match col {
        Column::EntireLine => Column::EntireLine,
        Column::Column(size, sep) => Column::Column(*size, String::from(sep))
    }
}

#[derive(PartialEq, Debug)]
pub enum FilterType {
    NotIn,
    AlsoIn,
    Duplicates
}
