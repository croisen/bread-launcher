use anyhow::{Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

use crate::account::{Account, AccountType};

pub fn login(
    cl: Client,
    luuid: impl AsRef<str>,
    name: impl AsRef<str>,
    pass: impl AsRef<str>,
) -> Result<Account> {
    let login = LoginJson::new(&luuid, &name, &pass);
    let res = cl
        .post("https://authserver.mojang.com/authenticate")
        .json(&login)
        .send()?;

    let stat = res.status();
    if !stat.is_success() {
        let bytes = res.bytes()?;
        let error = from_slice::<ErroredJson>(&bytes)?;
        log::error!("Error: {}", error.error);
        log::error!("Cause: {:?}", error.cause);
        log::error!("Long: {}", error.error_cause);
        bail!("Mojang auth errored out check the logs");
    }

    let bytes = res.bytes()?;
    let loginres = from_slice::<LoginResponse>(&bytes)?;
    let acc = Account {
        name: loginres.profile.name.as_str().into(),
        uuid: loginres.profile.id.as_str().into(),
        token: loginres.access_token.as_str().into(),
        account_type: AccountType::Mojang,
    };

    Ok(acc)
}

pub fn relogin(acc: &mut Account, cl: Client, luuid: impl AsRef<str>) -> Result<()> {
    let relogin = ReloginJson::new(acc, &luuid);
    let res = cl
        .post("https://authserver.mojang.com/refresh")
        .json(&relogin)
        .send()?;

    let stat = res.status();
    if !stat.is_success() {
        let bytes = res.bytes()?;
        let error = from_slice::<ErroredJson>(&bytes)?;
        log::error!("Error: {}", error.error);
        log::error!("Cause: {:?}", error.cause);
        log::error!("Long: {}", error.error_cause);
        bail!("Mojang auth errored out check the logs");
    }

    let bytes = res.bytes()?;
    let reloginres = from_slice::<ReloginResponse>(&bytes)?;
    acc.name = reloginres.profile.name.as_str().into();
    acc.uuid = reloginres.profile.id.as_str().into();
    acc.token = reloginres.access_token.as_str().into();

    Ok(())
}

#[derive(Default, Debug, Deserialize)]
struct ErroredJson {
    error: String,
    cause: Option<String>,
    #[serde(rename = "errorCause")]
    error_cause: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct LoginJson {
    agent: LoginAgent,
    username: String,
    password: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "requestUser")]
    request_user: bool,
}

impl LoginJson {
    fn new(luuid: impl AsRef<str>, name: impl AsRef<str>, pass: impl AsRef<str>) -> Self {
        Self {
            username: name.as_ref().into(),
            password: pass.as_ref().into(),
            client_token: luuid.as_ref().into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginAgent {
    name: String,
    version: usize,
}

impl Default for LoginAgent {
    fn default() -> Self {
        Self {
            name: "Minecraft".into(),
            version: 1,
        }
    }
}

#[derive(Default, Debug, Deserialize)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    /// Might change if they make accounts that have more than one profile
    /// as this struct should also contain a vector of profiles but I skipped
    /// it for now
    #[serde(rename = "selectedProfile")]
    profile: LoginProfile,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct LoginProfile {
    name: String,
    id: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct ReloginJson {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "selectedProfile")]
    profile: LoginProfile,
    #[serde(rename = "requestUser")]
    request_user: bool,
}

impl ReloginJson {
    fn new(acc: &Account, luuid: impl AsRef<str>) -> Self {
        Self {
            access_token: acc.token.as_ref().into(),
            client_token: luuid.as_ref().into(),
            request_user: false,
            profile: LoginProfile {
                name: acc.name.as_ref().into(),
                id: acc.uuid.as_ref().into(),
            },
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct ReloginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "selectedProfile")]
    profile: LoginProfile,
}
