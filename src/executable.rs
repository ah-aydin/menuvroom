#[derive(Debug, Clone)]
pub struct Executable {
    pub file_path: String,
    display_name_index: usize,
}

impl Executable {
    pub fn new(file_path: String) -> Self {
        Self {
            display_name_index: file_path.rfind('/').unwrap() + 1,
            file_path,
        }
    }

    pub fn get_display_name(&self) -> &str {
        &self.file_path[self.display_name_index..]
    }
}
