mod error;
mod lexer;
mod token;

use lexer::Lexer;
use std::fs;

fn main() {
    println!("Hello, Pyzza!");

    // Test the lexer with the hello_world.gem example
    let example_path = "example/hello_world.gem";
    match fs::read_to_string(example_path) {
        Ok(content) => {
            println!("Tokenizing: {}", example_path);
            let mut lexer = Lexer::new(content);
            match lexer.tokenize() {
                Ok(tokens) => {
                    println!("Tokens:");
                    for (i, token) in tokens.iter().enumerate() {
                        println!("  {}: {:?}", i, token);
                    }
                }
                Err(e) => {
                    println!("Lexer error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error reading file {}: {}", example_path, e);
        }
    }
}
