use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};

use anyhow::{Result, bail};
use crypto::digest::Digest;
use crypto::sha1::Sha1;

pub fn compare_sha1(expected: impl AsRef<str>, source: impl AsRef<[u8]>) -> Result<()> {
    let mut sha1 = Sha1::new();
    sha1.input(source.as_ref());
    let res = sha1.result_str();
    if res.eq_ignore_ascii_case(expected.as_ref()) {
        bail!(SHA1Mismatch::new(expected, res));
    }

    Ok(())
}

#[derive(Debug)]
pub struct SHA1Mismatch {
    expected: String,
    gotten: String,
}

impl SHA1Mismatch {
    pub fn new(expected: impl AsRef<str>, gotten: impl AsRef<str>) -> Self {
        Self {
            expected: expected.as_ref().into(),
            gotten: gotten.as_ref().into(),
        }
    }
}

impl Display for SHA1Mismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let expected = &self.expected;
        let gotten = &self.gotten;
        write!(f, "SHA1 mismatch, expected: {expected}, gotten: {gotten}")
    }
}
