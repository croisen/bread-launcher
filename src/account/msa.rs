use anyhow::{Result, bail};
use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use serde_json::from_slice;

use crate::account::Account;

pub fn login(
    _cl: Client,
    _luuid: impl AsRef<str>,
    _name: impl AsRef<str>,
    _pass: impl AsRef<str>,
) -> Result<Account> {
    bail!("Unimplemented");
}
