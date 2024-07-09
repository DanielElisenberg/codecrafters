use std::{env, path::PathBuf};

pub fn get_path() -> Vec<PathBuf> {
    let path_string = match env::var("PATH") {
        Ok(path_string) => path_string,
        Err(e) => panic!("{}", e),
    };
    path_string.split(":").map(PathBuf::from).collect()
}

pub fn get_current_dir() -> PathBuf {
    env::current_dir().unwrap()
}

pub fn set_current_dir(path: &PathBuf) {
    env::set_current_dir(path).unwrap();
}

pub fn get_home_dir() -> PathBuf {
    match env::var("HOME") {
        Ok(home) => PathBuf::from(home),
        Err(e) => panic!("{}", e),
    }
}
