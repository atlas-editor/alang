use std::io::{self, Write};

use crate::lex::Lexer;

const HEADER: &str = "alang 0.1";
const PROMPT: &str = "a> ";

pub fn start() {
    println!("{}", HEADER);

    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input!");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            break;
        }

        let l = Lexer::new(input.as_bytes());

        for tok in l {
            println!("{:?}", tok);
        }
    }
}
