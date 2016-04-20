extern crate regex;

use std::collections::HashMap;

#[derive(Debug)]
struct Schema {
    definitions: HashMap<String, HashMap<String, Column>>
}

impl Schema {
    fn read(filename: &str) -> Self {
        use std::fs::File;
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::path::Path;
        use regex::Regex;

        let path = Path::new(&filename);
        let mut defs = HashMap::new();

        if let Ok(file) = File::open(&path) {
            let reader      = BufReader::new(file);
            let table_re    = Regex::new("create_table \"(\\w+)\"").unwrap();
            let column_re   = Regex::new("t\\.(\\w+)\\s+\"(\\w+)\"(,\\s+(.+))?").unwrap();
            let mut table = "".into();

            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some(captures) = table_re.captures(&line) {
                        table = String::from(captures.at(1).unwrap());
                    } else if let Some(captures) = column_re.captures(&line) {
                        let col = Column {
                            table:   table.clone(),
                            kind:    String::from(captures.at(1).unwrap()),
                            name:    String::from(captures.at(2).unwrap()),
                            options: String::from(captures.at(4).unwrap_or("".into()))
                        };
                        let mut cols = defs.entry(table.clone()).or_insert(HashMap::new());
                        cols.insert(col.name.clone(), col);
                    } else {
                        // println!("REJ: {}", line);
                    }
                }
            }
            Schema { definitions: defs }
        } else {
            panic!("Cannot read file: {}", filename);
        }
    }

    fn repair(&self) {
        for (_, cols) in &self.definitions {
            for (_, col) in cols {
                if let Some(stmt) = col.fix_statement() { println!("{}", stmt); }
            }
        }
    }

    fn diff(&self, other: &Schema) {
        for (table_name, cols) in &self.definitions {
            if let Some(table) = other.definitions.get(table_name) {
                for (name, col) in cols {
                    if let Some(other_col) = table.get(name) {
                        col.diff(&other_col);
                    } else {
                        println!("Column '{}.{}' not found in schema #2", table_name, name);
                    }
                }
            } else {
                println!("Table '{}' not found in schema #2", table_name);
            }
        }

    }
}

#[derive(Debug)]
struct Column {
    table:   String,
    kind:    String,
    name:    String,
    options: String
}

impl Column {
    fn fix_statement(&self) -> Option<String> {
        match self.kind.as_ref() {
            "string" => if self.options.find("limit: 255").is_some() {
                Some(format!("ALTER TABLE \"{}\" ALTER COLUMN \"{}\" SET DATA TYPE character varying;", self.table, self.name))
            } else {
                None
            },
            _ => None
        }
    }

    fn diff(&self, other: &Column) {
        if self.kind != other.kind {
            println!("Mismatching types for '{}.{}': '{}' vs. '{}'", self.table, self.name, self.kind, other.kind);
        } else if self.options != other.options {
            println!("Mismatching options for '{}.{}':\n  '{}'\n  vs.\n  '{}'", self.table, self.name, self.options, other.options);
        }
    }

    fn to_string(&self) -> String {
        format!("{} '{}', {}", self.kind, self.name, self.options)
    }
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut buf = String::new();
        for (table_name, cols) in &self.definitions {
            buf.push_str(&format!("{}\n", table_name));
            for (_, col) in cols {
                buf.push_str(&format!("\t{}\n", col.to_string()));
            }
        }
        write!(f, "{}", buf)
    }
}

fn main() {
    use std::env::args;

    if args().nth(1).unwrap() == "dump" {
        let schema = Schema::read(&args().nth(2).unwrap());
        println!("{}", schema);
    } else if args().nth(1).unwrap() == "repair" {
        let schema = Schema::read(&args().nth(2).unwrap());
        schema.repair();
    } else if args().nth(1).unwrap() == "diff" {
        let schema1 = Schema::read(&args().nth(2).unwrap());
        let schema2 = Schema::read(&args().nth(3).unwrap());
        schema1.diff(&schema2);
    } else {
        panic!("Usage:\n  fix_schema repair schema.rb\n  fix_schema diff schema1.rb schema2.rb")
    }
}
