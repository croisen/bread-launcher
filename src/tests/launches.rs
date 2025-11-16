use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::mpmc::channel as mchannel;
use std::sync::mpsc::channel as schannel;

use crate::account::{Account, AccountType};
use crate::init::{get_instancedir, init_logs, init_reqwest};
use crate::loaders::forge::{Forge, download_forge_json};
use crate::loaders::minecraft::Minecraft;
use crate::tests::downloads::{test_forge_versions_download, test_minecraft_versions_download};

static RAM: usize = 3072;
static ACC: (&str, &str, &str, AccountType) = ("Croisen", "uuid?", "0", AccountType::Offline);

#[test]
fn test_minecraft_launch() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let _ = init_logs();
    let id = get_instancedir();

    let cl = init_reqwest();
    assert!(cl.is_ok(), "{:#?}", cl.unwrap_err());
    let cl = cl.unwrap();

    println!("Launching from latest version metadata");
    let ver = test_minecraft_versions_download();
    let m = Minecraft::new(id.join("test-latest"), &ver.id);
    assert!(m.is_ok(), "{:#?}", m.unwrap_err());
    let m = m.unwrap();

    let (stx, _srx) = schannel();
    let (_mtx, mrx) = mchannel();
    let s = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));

    let r = m.download(cl.clone(), s.clone(), stx.clone(), mrx.clone());
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());
    let r = m.run(
        RAM,
        Arc::new(Account {
            name: ACC.0.into(),
            uuid: ACC.1.into(),
            token: ACC.2.into(),
            account_type: ACC.3,
        }),
    );

    assert!(r.is_ok(), "{:#?}", r.unwrap_err());
}

#[test]
fn test_forge_launch() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let _ = init_logs();
    let id = get_instancedir();

    let cl = init_reqwest();
    assert!(cl.is_ok(), "{:#?}", cl.unwrap_err());
    let cl = cl.unwrap();

    let fvm = test_forge_versions_download();

    let mver = test_minecraft_versions_download();
    let fver = &fvm.versions[mver.id.as_ref()][0];

    let r = download_forge_json(cl.clone(), &mver.id, fver);
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());

    let forge = Forge::new(id.join("test-latest-forge"), &mver.id, fver);
    assert!(forge.is_ok(), "{:#?}", forge.unwrap_err());
    let forge = forge.unwrap();

    let (stx, _srx) = schannel();
    let (_mtx, mrx) = mchannel();
    let s = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));

    let r = forge.download(cl.clone(), s.clone(), stx.clone(), mrx.clone());
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());
    let r = forge.run(
        RAM,
        Arc::new(Account {
            name: ACC.0.into(),
            uuid: ACC.1.into(),
            token: ACC.2.into(),
            account_type: ACC.3,
        }),
    );

    assert!(r.is_ok(), "{:#?}", r.unwrap_err());
}
