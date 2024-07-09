use std::io::{self, Write};

mod builtin;
mod environment;
mod execute;

fn main() {
    loop {
        // Start REPL
        print!("$ ");
        io::stdout().flush().unwrap();

        // Parse user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let split_input = input.trim().split(" ").collect::<Vec<&str>>();
        let command = split_input[0];
        let arguments = split_input[1..].to_vec();

        // Evaluate parsed input
        match (command, arguments) {
            ("exit", args) => builtin::exit(args),
            ("echo", args) => builtin::echo(args),
            ("pwd", args) => builtin::pwd(args),
            ("cd", args) => builtin::cd(args),
            ("type", args) => builtin::type_builtin(args),
            (command, args) => match execute::run_binary(command, args) {
                Ok(_) => {}
                Err(_) => {
                    println!("{}: command not found", input.trim());
                }
            },
        };
    }
}
