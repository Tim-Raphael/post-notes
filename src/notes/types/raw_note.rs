use std::ffi;

pub struct RawNote {
    pub file_name: ffi::OsString,
    pub content: Vec<u8>,
}

impl RawNote {
    pub fn new(file_name: ffi::OsString, content: Vec<u8>) -> Self {
        Self { file_name, content }
    }
}

impl From<(ffi::OsString, Vec<u8>)> for RawNote {
    fn from(value: (ffi::OsString, Vec<u8>)) -> Self {
        Self::new(value.0, value.1)
    }
}
