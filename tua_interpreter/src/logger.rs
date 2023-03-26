use crate::prelude::*;



pub struct Logger {
    pub name: String,
    pub data: Vec<(usize, String)>,
}

impl Logger {

    pub fn new (name: impl Into<String>) -> Logger {
        let name = name.into();
        Self {
            name,
            data: vec!(),
        }
    }

    pub fn logln (&mut self, input: impl Into<String>) {
        let input = input.into();
        self.data.push((0, input));
    }

    pub fn log (&mut self, input: impl Into<String>) {
        let input = input.into();
        self.data.last_mut().unwrap().1 += &input;
    }

    pub fn join (&mut self, other: Logger) {
        self.data.push((0, String::from("[from: ") + &other.name + "]"));
        for (tab_level, line) in other.data {
            self.data.push((tab_level + 1, line));
        }
    }

    pub fn print_all (&self) {
        for (tab_level, line) in self.data.iter() {
            let mut output = String::new();
            for i in 0 .. *tab_level {
                output += "    ";
            }
            output += line;
            println!("{output}");
        }
    }

}
