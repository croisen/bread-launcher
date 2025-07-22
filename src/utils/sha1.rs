use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use anyhow::Result;
use crypto::digest::Digest;
use crypto::sha1::Sha1;

pub fn compare_sha1(expected: impl AsRef<str>, source: &[u8]) -> Result<()> {
    let res = regular_sha1(source);
    if res.eq_ignore_ascii_case(expected.as_ref()) {
        Ok(())
    } else {
        Err(SHA1Mismatch::new(expected.as_ref().to_string(), res).into())
    }
}

fn regular_sha1(data: &[u8]) -> String {
    let mut sha1 = Sha1::new();
    sha1.input(data);
    sha1.result_str()
}

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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let expected = &self.expected;
        let gotten = &self.gotten;
        write!(f, "SHA1 mismatch, expected: {expected}, gotten: {gotten}")
    }
}
