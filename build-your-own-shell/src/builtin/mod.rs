use std::{path::PathBuf, process};

use crate::environment;

pub fn exit(args: Vec<&str>) {
    process::exit(args[0].parse().expect("Exit code has to be an integer"))
}

pub fn echo(args: Vec<&str>) {
    let output = args.join(" ");
    println!("{output}");
}

pub fn pwd(_args: Vec<&str>) {
    let current_dir_path = environment::get_current_dir();
    println!("{}", current_dir_path.to_str().unwrap());
}

pub fn cd(args: Vec<&str>) {
    match args[0] {
        "./" => {}
        "~" => {
            let home_directory = environment::get_home_dir();
            if !home_directory.exists() {
                println!(
                    "cd: {}: No such file or directory",
                    home_directory.to_str().unwrap()
                );
            } else {
                environment::set_current_dir(&home_directory)
            }
        }
        p if p.starts_with("/") => {
            let path = PathBuf::from(args[0]);
            if !path.exists() {
                println!("cd: {}: No such file or directory", path.to_str().unwrap());
            } else {
                environment::set_current_dir(&path)
            }
        }
        p if p.starts_with("~") => {
            let home_directory = environment::get_home_dir();
            let mut remaining_path = p.to_string();
            remaining_path.remove(0);
            let path = home_directory.join(PathBuf::from(remaining_path));
            if !path.exists() {
                println!("cd: {}: No such file or directory", path.to_str().unwrap());
            } else {
                environment::set_current_dir(&path)
            }
        }
        p => {
            let path = environment::get_current_dir().join(PathBuf::from(p));
            if !path.exists() {
                println!("cd: {}: No such file or directory", path.to_str().unwrap());
            } else {
                environment::set_current_dir(&path);
            }
        }
    }
}

pub fn type_builtin(args: Vec<&str>) {
    let output: String = match args[0] {
        "echo" | "exit" | "type" | "pwd" | "cd" => format!("{} is a shell builtin", args[0]),
        binary_name => {
            let mut bin_output: Option<String> = None;
            for path in environment::get_path() {
                let binary_path = path.join(binary_name);
                if binary_path.exists() {
                    bin_output = Some(format!("{} is {}", args[0], binary_path.to_str().unwrap()));
                };
            }
            match bin_output {
                Some(found_path_output) => found_path_output,
                None => format!("{}: not found", args[0]),
            }
        }
    };
    println!("{output}");
}
