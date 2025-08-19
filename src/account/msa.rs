#![allow(unused_imports)]
use std::net::TcpListener;

use anyhow::{Result, bail};
use oauth2::basic::BasicClient;
use oauth2::reqwest::blocking::Client as OAuthClient;
use oauth2::reqwest::redirect::Policy;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use opener::open_browser as webopener;
use reqwest::blocking::Client;

use crate::account::Account;
use crate::init::OAUTH_CLIENT_ID;

pub fn login(
    _cl: Client,
    _luuid: impl AsRef<str>,
    _name: impl AsRef<str>,
    _pass: impl AsRef<str>,
) -> Result<Account> {
    // I need an Azure account with an active subscription???
    bail!("Unimplemented");
}
