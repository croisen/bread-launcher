use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::spawn;

use egui::Context;
use parking_lot::Mutex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::account::{Account, AccountType};
use crate::utils::message::Message;
use crate::utils::{ShowWindow, WindowData};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountWin {
    acc_type: AccountType,

    username: String,
    password: String,
    show_pass: bool,

    #[serde(skip, default = "channel::<Message>")]
    channel: (Sender<Message>, Receiver<Message>),
}

impl Default for AccountWin {
    fn default() -> Self {
        Self {
            acc_type: AccountType::Offline,

            username: String::new(),
            password: String::new(),
            show_pass: false,

            channel: channel::<Message>(),
        }
    }
}

impl ShowWindow for AccountWin {
    fn show(
        &mut self,
        _mctx: Context,
        ctx: &Context,
        _show_win: Arc<AtomicBool>,
        data: WindowData,
        cl: Client,
    ) {
        let (accounts, account, luuid) = data;
        egui::SidePanel::right("Add Account - Side Panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Add Account");
                ui.separator();

                if ui.button("Add Offline").clicked() {
                    self.acc_type = AccountType::Offline;
                }

                ui.disable();

                if ui.button("Add Legacy").clicked() {
                    self.acc_type = AccountType::Legacy;
                }

                if ui.button("Add Mojang").clicked() {
                    self.acc_type = AccountType::Mojang;
                }

                if ui.button("Add Microsoft").clicked() {
                    self.acc_type = AccountType::Msa;
                }

                ui.separator();
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let ulabel = ui.label("Username / Email");
                let default = ui.style().spacing.text_edit_width;
                let padding = ui.style().spacing.button_padding.x * 2.0;
                ui.style_mut().spacing.text_edit_width = ui.available_width() - padding;
                ui.text_edit_singleline(&mut self.username)
                    .labelled_by(ulabel.id);
                ui.style_mut().spacing.text_edit_width = default;
            });

            ui.horizontal(|ui| {
                let plabel = ui.label("Password");
                let default = ui.style().spacing.text_edit_width;
                let padding = ui.style().spacing.button_padding.x * 2.0;
                ui.style_mut().spacing.text_edit_width = ui.available_width() - padding;
                let pass_text =
                    egui::TextEdit::singleline(&mut self.password).password(!self.show_pass);
                ui.add(pass_text).labelled_by(plabel.id);
                ui.style_mut().spacing.text_edit_width = default;
            });

            ui.with_layout(
                egui::Layout::top_down(egui::Align::TOP).with_cross_align(egui::Align::RIGHT),
                |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut self.show_pass, "Show Password");

                        if ui
                            .button(format!("Add {} Account", self.acc_type))
                            .clicked()
                        {
                            let username = self.username.clone();
                            let password = self.password.clone();
                            let accounts = accounts.clone();
                            let account = account.clone();
                            let acc_type = self.acc_type;
                            let cl = cl.clone();
                            let client_uuid = luuid.clone();
                            spawn(move || {
                                let luuid = client_uuid.downcast_ref::<String>().unwrap();
                                let acc = match acc_type {
                                    AccountType::Legacy => {
                                        Account::new_legacy(cl, luuid, username, password)
                                    }
                                    AccountType::Mojang => {
                                        Account::new_mojang(cl, luuid, username, password)
                                    }
                                    AccountType::Msa => {
                                        Account::new_msa(cl, luuid, username, password)
                                    }
                                    AccountType::Offline => Ok(Account::new_offline(username)),
                                };

                                if acc.is_err() {
                                    log::error!("{:?}", acc.unwrap_err());
                                    return;
                                }

                                let acc = acc.unwrap();
                                let mut current =
                                    account.downcast_ref::<Mutex<Account>>().unwrap().lock();

                                *current = acc.clone();
                                accounts
                                    .downcast_ref::<Mutex<Vec<Account>>>()
                                    .unwrap()
                                    .lock()
                                    .push(acc.clone());
                            });
                        }
                    });
                },
            );

            ui.separator();

            ui.vertical_centered_justified(|ui| {
                ui.heading("Accounts");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut current = account.downcast_ref::<Mutex<Account>>().unwrap().lock();
                    let accs = accounts
                        .downcast_ref::<Mutex<Vec<Account>>>()
                        .unwrap()
                        .lock();

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
