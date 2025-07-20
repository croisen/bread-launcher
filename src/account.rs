use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::SystemTime;

use rand::{RngCore, rng};
use serde::{Deserialize, Serialize};
use uuid::Builder as UB;
use uuid::Version;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum AccountType {
    MSA,
    LEGACY,
    MOJANG,
    #[default]
    OFFLINE,
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let at = format!("{self:?}").to_ascii_lowercase();

        write!(f, "{at}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: Arc<str>,
    pub uuid: Arc<str>,
    pub token: Arc<str>,
    pub account_type: AccountType,
    pub selected: bool,
}

impl Account {}

impl Default for Account {
    fn default() -> Self {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let mut rb: [u8; 10] = [0; 10];
        rng().fill_bytes(&mut rb);

        let uuid = UB::from_unix_timestamp_millis(ts.as_millis().try_into().unwrap(), &rb)
            .with_version(Version::SortRand)
            .into_uuid()
            .to_string();
        Self {
            name: Arc::from("croisen"),
            uuid: Arc::from(uuid),
            token: Arc::from("0"), // I ain't putting a real token here
            account_type: AccountType::OFFLINE,
            selected: false,
        }
    }
}
