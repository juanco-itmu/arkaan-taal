mod ast;
mod bytecode;
mod compiler;
mod lexer;
mod parser;
mod token;
mod value;
mod vm;

use std::env;
use std::fs;
use std::io::{self, Write};

use compiler::Compiler;
use lexer::Lexer;
use parser::Parser;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Gebruik: arkaan [lêer.ark]");
            std::process::exit(64);
        }
    }
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Kon nie lêer lees nie: {}", e);
            std::process::exit(66);
        }
    };

    if let Err(e) = run(&source) {
        eprintln!("Fout: {}", e);
        std::process::exit(70);
    }
}

fn repl() {
    println!("Arkaan v0.1.0 - 'n Afrikaanse programmeertaal");
    println!("Tik 'verlaat' om te stop.\n");

    loop {
        print!("arkaan> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }

        let line = line.trim();
        if line == "verlaat" {
            println!("Totsiens!");
            break;
        }

        if line.is_empty() {
            continue;
        }

        if let Err(e) = run(line) {
            eprintln!("Fout: {}", e);
        }
    }
}

fn run(source: &str) -> Result<(), String> {
    // Lexing
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    // Compiling
    let mut compiler = Compiler::new();
    let (chunk, functions) = compiler.compile(statements)?;

    // Executing
    let mut vm = VM::new(chunk, functions);
    vm.run()
}
