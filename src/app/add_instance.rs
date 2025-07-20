use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use egui::Context;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::instance::{InstanceLoader, Instances};
use crate::utils::ShowWindow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInstance {
    name: String,
    group: String,
    version: Arc<str>,
    loader: InstanceLoader,
}

impl Default for AddInstance {
    fn default() -> Self {
        Self {
            name: String::new(),
            group: String::new(),
            version: Arc::from("0"),
            loader: InstanceLoader::Vanilla,
        }
    }
}

impl ShowWindow for AddInstance {
    fn show(
        &mut self,
        mctx: egui::Context,
        ctx: &egui::Context,
        show_win: Arc<AtomicBool>,
        data: Arc<dyn Any>,
    ) {
        let instances = data.downcast_ref::<Mutex<Instances>>().unwrap();
    }
}
