use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct SHA1Mismatch {
    expected: String,
    gotten: String,
}

impl SHA1Mismatch {
    pub fn new(expected: String, gotten: String) -> Self {
        Self { expected, gotten }
    }
}

impl Error for SHA1Mismatch {}
impl Display for SHA1Mismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let expected = &self.expected;
        let gotten = &self.gotten;
        write!(f, "SHA1 mismatch, expected: {expected}, gotten: {gotten}")
    }
}
