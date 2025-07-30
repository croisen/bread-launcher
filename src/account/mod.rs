use std::fmt::{Display, Formatter};
use std::sync::Arc;

use anyhow::Result;
use rand::{RngCore, rng};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use uuid::Builder as UB;

mod mojang;
mod msa;

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountType {
    _LEGACY,
    MOJANG,
    MSA,
    #[default]
    OFFLINE,
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let at = format!("{self:?}").to_ascii_lowercase();

        write!(f, "{at}")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub name: Arc<str>,
    pub uuid: Arc<str>,
    pub token: Arc<str>,
    pub account_type: AccountType,
}

impl Account {
    pub fn new_offline(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().into(),
            ..Default::default()
        }
    }

    pub fn new_mojang(
        cl: Client,
        luuid: impl AsRef<str>,
        name: impl AsRef<str>,
        pass: impl AsRef<str>,
    ) -> Result<Self> {
        mojang::login(cl, luuid, name, pass)
    }

    pub fn new_msa(
        cl: Client,
        luuid: impl AsRef<str>,
        name: impl AsRef<str>,
        pass: impl AsRef<str>,
    ) -> Result<Self> {
        msa::login(cl, luuid, name, pass)
    }

    pub fn relogin(&mut self, cl: Client, luuid: impl AsRef<str>) -> Result<()> {
        match self.account_type {
            AccountType::_LEGACY => {}
            AccountType::MOJANG => {
                mojang::relogin(self, cl, luuid)?;
            }
            AccountType::MSA => {}
            AccountType::OFFLINE => {}
        }

        Ok(())
    }
}

impl Default for Account {
    fn default() -> Self {
        let mut rb = [0; 16];
        rng().fill_bytes(&mut rb);
        let uuid = UB::from_random_bytes(rb).into_uuid().to_string();

        Self {
            name: Arc::from("croisen"),
            uuid: Arc::from(uuid),
            token: Arc::from("0"), // I ain't putting a real token here
            account_type: AccountType::OFFLINE,
        }
    }
}
