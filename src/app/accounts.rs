use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::Context;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::account::Account;
use crate::utils::ShowWindow;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AccountWin {
    acc_to_add: Account,
}

impl ShowWindow for AccountWin {
    fn show(
        &mut self,
        mctx: Context,
        ctx: &Context,
        show_win: Arc<AtomicBool>,
        data: Arc<dyn Any>,
    ) {
    }
}
