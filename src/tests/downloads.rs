use std::fs::write;

use crate::init::{get_instancedir, init_reqwest};
use crate::loaders::forge::ForgeVersionManifest;
use crate::loaders::minecraft::MVOrganized;

#[test]
fn test_minecraft_versions_download() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let id = get_instancedir();

    let cl = init_reqwest();
    assert!(cl.is_ok(), "{:#?}", cl.unwrap_err());
    let cl = cl.unwrap();

    let mut mvo = MVOrganized::default();
    let r = mvo.renew(cl.clone());
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());

    println!("MVO Releases  : {}", mvo.release.len());
    println!("MVO Snapshots : {}", mvo.snapshot.len());
    println!("MVO Betas     : {}", mvo.beta.len());
    println!("MVO Alphas    : {}", mvo.alpha.len());

    assert!(mvo.release.len() > 0);
    assert!(mvo.snapshot.len() > 0);
    assert!(mvo.beta.len() > 0);
    assert!(mvo.alpha.len() > 0);

    println!("Downloading latest version metadata");
    let v = &mvo.release[0];
    let r = v.download(cl.clone());
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());

    let r = write(id.join("latest-vanilla-test.txt"), v.id.as_ref());
    assert!(r.is_ok(), "{:#?}", r.unwrap_err());
}

#[test]
fn test_forge_versions_download() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };

    let cl = init_reqwest();
    assert!(cl.is_ok(), "{:#?}", cl.unwrap_err());
    let cl = cl.unwrap();

    let fvm = ForgeVersionManifest::new(cl.clone());
    assert!(fvm.is_ok(), "{:#?}", fvm.unwrap_err());
    let fvm = fvm.unwrap();

    for (k, v) in &fvm.recommends.promos {
        println!("{k} : {v:?}");
    }
}
