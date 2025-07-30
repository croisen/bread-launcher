use anyhow::{Result, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

use crate::account::Account;

pub fn login(
    cl: Client,
    luuid: impl AsRef<str>,
    name: impl AsRef<str>,
    pass: impl AsRef<str>,
) -> Result<Account> {
    bail!("Unimplemented");
}
