use core::fmt;
use std::collections::HashSet;

use calamine::{Data, Range};
use csv::Reader;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Values {
    Nil,
    String(String),
    Number(String),
    Boolean(bool),
}

impl fmt::Display for Values {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Values::Nil => write!(f, "Nil"),
            Values::String(s) => write!(f, "{}", s),
            Values::Number(n) => write!(f, "{}", n),
            Values::Boolean(b) => write!(f, "{}", b),
        }
    }
}

pub struct Sheet {
    pub name: String,
    pub types: Vec<(String, String)>,
    pub sheet: Vec<Vec<Values>>,
}

impl Sheet {
    pub fn excel(name: String, range: Range<Data>) -> Self {
        let mut types = vec![];
        let mut sheet = vec![];

        for row in 0..range.get_size().1 {
            types.push((
                match range.get((0, row)).unwrap() {
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::String(s) => s.to_owned(),
                    Data::Bool(_) => panic!("key at {} is a boolean", row + 1),
                    Data::Empty => panic!("key at {} is nil", row + 1),

                    Data::Error(e) => panic!("{}", e),

                    Data::DateTime(d) => d.to_string(),
                    Data::DateTimeIso(d) | Data::DurationIso(d) => d.to_owned(),
                },
                match range.get((1, row)).unwrap() {
                    Data::Int(_) | Data::Float(_) => "number".to_owned(),
                    Data::String(_) => "string".to_owned(),
                    Data::Bool(_) => "boolean".to_owned(),
                    Data::Empty => "nil".to_owned(),

                    Data::Error(e) => panic!("{}", e),

                    Data::DateTime(_) | Data::DateTimeIso(_) | Data::DurationIso(_) => {
                        "string".to_owned()
                    }
                },
            ));
        }

        for column in 1..range.get_size().0 {
            let mut sheet_row = vec![];
            for row in 0..range.get_size().1 {
                sheet_row.push(match range.get((column, row)).unwrap() {
                    Data::Int(i) => Values::Number(i.to_string()),
                    Data::Float(f) => Values::Number(f.to_string()),
                    Data::String(s) => Values::String(s.to_owned()),
                    Data::Bool(b) => Values::Boolean(b.to_owned()),
                    Data::Empty => Values::Nil,

                    Data::Error(e) => panic!("{e}"),

                    Data::DateTime(d) => Values::String(d.to_string()),
                    Data::DateTimeIso(d) | Data::DurationIso(d) => Values::String(d.to_owned()),
                })
            }
            sheet.push(sheet_row);
        }

        Self { name, types, sheet }
    }

    pub fn csv(tabs: bool, name: String, csv: String) -> Self {
        if tabs {
            unimplemented!("tsv not implemented")
        }

        let mut reader = Reader::from_reader(csv.as_bytes());
        let headers = reader.headers().unwrap().clone();
        let row_count = reader.records().count();
        let mut types = vec![];

        for (i, header) in headers.iter().enumerate() {
            let mut reader = Reader::from_reader(csv.as_bytes());
            let mut header_types = HashSet::new();

            for record in reader.records() {
                let record = record.unwrap();
                let field = record.get(i).unwrap();
                let value = parse_value(field);
                header_types.insert(value);
            }

            let header_type = determine_type(&header_types, row_count);
            types.push((header.to_string(), header_type));
        }

        let mut reader = Reader::from_reader(csv.as_bytes());
        let sheet: Vec<Vec<Values>> = reader
            .records()
            .map(|record| {
                record
                    .unwrap()
                    .iter()
                    .map(|field| parse_value(field))
                    .collect()
            })
            .collect();

        Self { name, types, sheet }
    }
}

fn parse_value(field: &str) -> Values {
    if let Ok(int_value) = field.parse::<i64>() {
        Values::Number(int_value.to_string())
    } else if let Ok(float_value) = field.parse::<f64>() {
        Values::Number(float_value.to_string())
    } else if let Ok(bool_value) = field.parse::<bool>() {
        Values::Boolean(bool_value)
    } else if field.is_empty() {
        Values::Nil
    } else {
        Values::String(field.to_string())
    }
}

fn determine_type(header_types: &HashSet<Values>, row_count: usize) -> String {
    if header_types.len() == 1 {
        match header_types.iter().next().unwrap() {
            Values::Nil => "nil".to_owned(),
            Values::String(string_value) => format!("'{}'", string_value),
            Values::Number(_) => "number".to_owned(),
            Values::Boolean(bool_value) => bool_value.to_string(),
        }
    } else {
        let (mut only_numbers, mut only_boolean, mut only_string, mut has_nil) =
            (true, true, true, false);
        let mut string_count = 0;
        for header_type in header_types.iter() {
            match header_type {
                Values::Number(_) => {
                    only_string = false;
                    only_boolean = false;
                }
                Values::Boolean(_) => {
                    only_string = false;
                    only_numbers = false;
                }
                Values::String(_) => {
                    only_numbers = false;
                    only_boolean = false;
                    string_count += 1;
                }
                Values::Nil => has_nil = true,
                _ => {
                    only_string = false;
                    only_numbers = false;
                    only_boolean = false;
                }
            }
        }
        if only_string && string_count < row_count {
            let mut string_header_type = String::new();
            for header_type in header_types.iter() {
                match header_type {
                    Values::String(string_value) => {
                        string_header_type += &format!("'{}' | ", string_value);
                    }
                    _ => {}
                }
            }
            string_header_type = string_header_type.strip_suffix(" | ").unwrap().to_owned();
            string_header_type
        } else {
            let mut header_type = match (only_numbers, only_boolean) {
                (true, false) => "number".to_owned(),
                (false, true) => "boolean".to_owned(),
                _ => "string".to_owned(),
            };
            if has_nil {
                header_type += " | nil";
            }
            header_type
        }
    }
}
