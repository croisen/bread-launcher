use std::any::Any;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use anyhow::anyhow;
use egui::Context;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::account::{Account, AccountType};
use crate::utils::ShowWindow;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AccountWin {
    acc_type: AccountType,

    username: String,
    password: String,
}

impl ShowWindow for AccountWin {
    fn show(
        &mut self,
        _mctx: Context,
        ctx: &Context,
        _show_win: Arc<AtomicBool>,
        accounts: Arc<dyn Any + Sync + Send>,
        account: Arc<dyn Any + Sync + Send>,
        luuid: Arc<dyn Any + Sync + Send>,
        cl: Client,
    ) {
        egui::SidePanel::right("Add Account - Side Panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Add Account");
                ui.separator();

                if ui.button("Add Offline").clicked() {
                    self.acc_type = AccountType::OFFLINE;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let label = ui.label("Username / Email");
                ui.text_edit_singleline(&mut self.username)
                    .labelled_by(label.id);
            });

            if self.acc_type != AccountType::OFFLINE {
                ui.horizontal(|ui| {
                    let label = ui.label("Password");
                    ui.text_edit_singleline(&mut self.password)
                        .labelled_by(label.id);
                });
            }

            egui::Sides::new().show(
                ui,
                |_ui| {},
                |ui| {
                    let username = self.username.clone();
                    let password = self.password.clone();
                    let accounts = accounts.clone();
                    let account = account.clone();
                    let acc_type = self.acc_type;
                    let cl = cl.clone();
                    let client_uuid = luuid.clone();
                    if ui.button("Add Account").clicked() {
                        spawn(move || {
                            let luuid = client_uuid.downcast_ref::<String>().unwrap();
                            let acc = match acc_type {
                                AccountType::_LEGACY => Err(anyhow!("Not implemented")),
                                AccountType::MOJANG => {
                                    Account::new_mojang(cl, luuid, username, password)
                                }
                                AccountType::MSA => Account::new_msa(cl, luuid, username, password),
                                AccountType::OFFLINE => Ok(Account::new_offline(username)),
                            };

                            if acc.is_err() {
                                log::error!("{:?}", acc.unwrap_err());
                                return;
                            }

                            let acc = acc.unwrap();
                            let mut current = account
                                .downcast_ref::<Mutex<Account>>()
                                .unwrap()
                                .lock()
                                .unwrap();
                            *current = acc.clone();
                            accounts
                                .downcast_ref::<Mutex<Vec<Account>>>()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .push(acc.clone());
                        });
                    }
                },
            );

            ui.separator();

            ui.vertical_centered_justified(|ui| {
                ui.heading("Accounts");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let accs = accounts
                        .downcast_ref::<Mutex<Vec<Account>>>()
                        .unwrap()
                        .lock()
                        .unwrap();
                    let mut current = account
                        .downcast_ref::<Mutex<Account>>()
                        .unwrap()
                        .lock()
                        .unwrap();

                    for acc in accs.iter() {
                        let selected = current.eq(acc);
                        let text = egui::RichText::new(format!(
                            "{:>10} | {} | {}",
                            acc.name, acc.uuid, acc.account_type
                        ))
                        .monospace();

                        let mut label = ui.selectable_label(selected, text);
                        if label.clicked() {
                            *current = acc.clone();
                            label.mark_changed();
                        }
                    }
                });
            });
        });
    }
}
