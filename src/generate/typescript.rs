use crate::{sheet::Values, Config};

struct Generator {
    config: Config,
    tabs: i32,
    buf: String,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tabs: 0,
            buf: String::new(),
        }
    }

    fn push(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn indent(&mut self) {
        self.tabs += 1;
    }

    fn dedent(&mut self) {
        self.tabs -= 1;
    }

    fn push_tabs(&mut self) {
        for _ in 0..self.tabs {
            self.push("\t");
        }
    }

    fn push_line(&mut self, s: &str) {
        self.push_tabs();
        self.push(s);
        self.push("\n");
    }

    fn push_type(&mut self) {
        self.push_line(format!("export interface {} {{", self.config.sheet.name).as_str());
        self.indent();

        for (key, type_name) in self.config.sheet.types.clone() {
            self.push_line(format!("{}: {},", key, type_name).as_str());
        }

        self.dedent();
        self.push_line("};");
    }

    fn push_objects(&mut self) {
        for column in self.config.sheet.sheet.clone() {
            self.push_line(format!("export const {} = {{", column[self.config.table_name as usize]).as_str());
            self.indent();

            for (i, row) in column.iter().enumerate() {
                if i == self.config.table_name.try_into().unwrap() { continue; }

                match row {
                    Values::Nil => self.push_line("undefined,"),
                    Values::String(s) => self.push_line(format!("{}: '{}',", self.config.sheet.types[i].0, s).as_str()),
                    Values::Number(n) => self.push_line(format!("{}: {},", self.config.sheet.types[i].0, n).as_str()),
                    Values::Boolean(b) => self.push_line(format!("{}: {},", self.config.sheet.types[i].0, b).as_str()),
                }
            }

            self.dedent();
            self.push_line("};");
            self.push("\n");
        }
    }

    pub fn generate(mut self) -> (String, String) {
        if !self.config.no_type {
            self.push_type();
            self.push("\n");
        }

        self.push_objects();

        (format!("{}.ts", self.config.sheet.name), self.buf)
    }
}

pub fn code(config: Config) -> (String, String) {
    Generator::new(config).generate()
}