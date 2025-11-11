use std::fs::read_to_string;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::mpmc::channel as mchannel;
use std::sync::mpsc::channel as schannel;
use std::thread::sleep;
use std::time::Duration;

use crate::account::{Account, AccountType};
use crate::init::{get_instancedir, init_reqwest};
use crate::loaders::minecraft::Minecraft;

static RAM: usize = 3072;
static ACC: (&str, &str, &str, AccountType) = ("Croisen", "uuid?", "0", AccountType::Offline);

#[test]
fn test_minecraft_launch() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let id = get_instancedir();

    let cl = init_reqwest();
    assert!(cl.is_ok(), "{:#?}", cl.unwrap_err());
    let cl = cl.unwrap();

    println!("Launching from latest version metadata");

    // May test_minecraft_versions_download be tested first before this
    // Though if it is run all at once then this fails
    // So we sleep first

    sleep(Duration::new(10, 0));
    let ver = read_to_string(id.join("latest-vanilla-test.txt"));
    assert!(ver.is_ok(), "{:#?}", ver.unwrap_err());
    let ver = ver.unwrap();

    let m = Minecraft::new(id.join("test-latest"), ver);
    assert!(m.is_ok(), "{:#?}", m.unwrap_err());
    let m = m.unwrap();

    let (stx, _srx) = schannel();
    let (_mtx, mrx) = mchannel();
    let s = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));

    let r = m.download_jre(cl.clone(), s.clone(), stx.clone(), mrx.clone());
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());
    let r = m.download_client(cl.clone(), s.clone(), stx.clone(), mrx.clone());
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());
    let r = m.download_assets(cl.clone(), s.clone(), stx.clone(), mrx.clone());
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
