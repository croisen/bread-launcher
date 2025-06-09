use std::fs::{read, write};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::{Compress, Compression, Decompress};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use tokio::runtime::Handle;

use crate::instance::Instances;
use crate::utils::ShowWindow;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreadLauncher {
    // TODO: Make a dedicated account struct later that will be passed
    // on to the run function of the Instance struct
    accounts: Vec<String>,

    // Forgot about groups huh, should I go for a btreemap with multiple
    // instances? that seems inefficient since it also has a version list
    // TODO: Revise the instances struct to include groups
    instances: Arc<Mutex<Instances>>,
    appdir: PathBuf,
    last_check: u64,

    #[serde(skip)]
    cl: Client,
}

impl BreadLauncher {
    pub fn new(cl: Client, appdir: impl AsRef<Path>, handle: &Handle) -> Result<Self> {
        let f = appdir.as_ref().join("save.blauncher");
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        if f.is_file() {
            let saved = read(&f)?;
            let mut decomp = vec![];
            let mut d = ZlibDecoder::new_with_decompress(&mut decomp, Decompress::new(true));
            d.write_all(saved.as_ref())?;
            let _ = d.finish()?;

            let mut de = Deserializer::from_slice(decomp.as_ref());
            let mut s = Self::deserialize(&mut de)?;

            // Re-download version manifest if 10 days has passed
            let r = Duration::new(10 * 24 * 60 * 60, 0);
            let since = now.saturating_sub(Duration::new(s.last_check, 0));
            let vm = appdir.as_ref().join("version_manifest_v2.json");
            let instances = appdir.as_ref().join("instances.json");
            if since.as_secs() >= r.as_secs() || !vm.exists() {
                log::info!("Checking for new versions as 10 days has passed since the last check or the version manifest just doesn't exist");
                handle.block_on(s.instances.lock().unwrap().renew_version(appdir.as_ref()))?;
            }

            s.cl = cl;
            Ok(s)
        } else {
            let i = handle.block_on(Instances::new(cl.clone(), appdir.as_ref()))?;
            Ok(Self {
                accounts: vec![],
                instances: Arc::new(Mutex::new(i)),
                appdir: appdir.as_ref().to_path_buf(),
                last_check: now.as_secs(),

                cl,
            })
        }
    }

    pub fn savefile(&self) -> Result<()> {
        let mut s = vec![];
        let mut comp = vec![];
        let mut se = Serializer::pretty(&mut s);
        self.serialize(&mut se)?;

        let mut c =
            ZlibEncoder::new_with_compress(&mut comp, Compress::new(Compression::best(), true));
        let _ = c.write_all(s.as_ref())?;
        let _ = c.finish()?;

        write(self.appdir.join("save.blauncher"), comp)?;
        Ok(())
    }

    fn show_window<W: ShowWindow + Send + Sync + 'static, T: Send + Sync + 'static>(
        &self,
        ctx: egui::Context,
        id: &str,
        win: Arc<Mutex<W>>,
        data: Arc<Mutex<T>>,
        show_window: Arc<AtomicBool>,
    ) {
        if show_window.load(Ordering::Relaxed) {
            let mctx = Arc::new(ctx.clone());
            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of(id),
                egui::ViewportBuilder::default().with_title(id),
                move |wctx, cls| {
                    assert!(
                        cls == egui::ViewportClass::Deferred,
                        "The backend doesn't support multiple viewports?"
                    );

                    win.lock()
                        .unwrap()
                        .show(wctx, mctx.clone(), data.clone(), show_window.clone());

                    if wctx.input(|i| i.viewport().close_requested()) {
                        show_window.store(false, Ordering::Relaxed)
                    }
                },
            );
        }
    }
}

impl eframe::App for BreadLauncher {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        log::info!("Saving egui state to {:?}", self.appdir.join("egui.ron"));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let f = self.appdir.join("save.blauncher");
        log::info!("Saving app state to {:?}", f);
        if let Err(e) = self.savefile() {
            log::error!("Error in saving app state to {:?}", f);
        }
    }

    fn update(&mut self, ctx: &egui::Context, _fr: &mut eframe::Frame) {
        egui::TopBottomPanel::top("main-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Add Instance").clicked() {};
                if ui.button("About").clicked() {};
            });
        });

        egui::SidePanel::right("main-side-panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui.button("Instance Info").clicked() {}
                if ui.button("Add Mods").clicked() {}
                if ui.button("Logs").clicked() {}
                if ui.button("Delete").clicked() {}

                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                    if ui.button("Start").clicked() {}
                    if ui.button("Start Offline").clicked() {}
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");
        });
    }
}
