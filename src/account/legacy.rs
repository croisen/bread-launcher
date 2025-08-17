use std::collections::HashMap;

use anyhow::{Result, bail};
use reqwest::blocking::Client;

use crate::account::{Account, AccountType};

pub fn login(
    cl: Client,
    _luuid: impl AsRef<str>,
    name: impl AsRef<str>,
    pass: impl AsRef<str>,
) -> Result<Account> {
    let mut form = HashMap::new();
    form.insert("user", name.as_ref());
    form.insert("password", pass.as_ref());
    form.insert("version", "13");

    let res = cl.post("https://login.minecraft.net").form(&form).send()?;
    let status = res.status();
    if !status.is_success() {
        bail!("minecraft.net login returned an error");
    }

    let bytes = res.bytes()?;
    let res_str = String::from_utf8_lossy(&bytes).to_string();
    if res_str == "Bad response" || res_str == "Bad login" {
        bail!(res_str);
    }

    let res_split = res_str.split(":").collect::<Vec<&str>>();
    let acc = Account {
        name: res_split[2].into(),
        uuid: res_split[4].into(),
        token: res_split[3].into(),
        account_type: AccountType::Legacy,
    };

    Ok(acc)
}

pub fn relogin(acc: &mut Account, cl: Client, _luuid: impl AsRef<str>) -> Result<()> {
    let mut form = HashMap::new();
    form.insert("name", acc.name.as_ref());
    form.insert("session", acc.token.as_ref());

    let res = cl
        .post("https://login.minecraft.net/session")
        .form(&form)
        .send()?;

    let status = res.status();
    if !status.is_success() {
        bail!("minecraft.net login returned an error");
    }

    let bytes = res.bytes()?;
    let res_str = String::from_utf8_lossy(&bytes).to_string();
    if res_str == "Bad response" || res_str == "Bad login" {
        bail!(res_str);
    }

    Ok(())
}
