pub fn read_file_to_string(file_path: String) -> Result<String, String> {
    if !std::path::Path::new(&file_path).exists() {
        return Err("File not Found".to_string());
    }
    return Ok(std::fs::read_to_string(file_path).unwrap());
}

pub fn write_file_to_string(file_path: String, contents: String) -> Result<(), std::io::Error> {
    std::fs::write(file_path, contents)
}
