#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Property {
    file_no: usize,
    getter: u32,
    setter: u32,
}

impl Property {
    pub fn new(file_no: usize, getter: u32, setter: u32) -> Property {
        Property {
            file_no,
            getter,
            setter,
        }
    }

    pub fn get_file_no(&self) -> usize {
        self.file_no
    }

    pub fn get_getter(&self) -> u32 {
        self.getter
    }

    pub fn get_setter(&self) -> u32 {
        self.setter
    }
}
