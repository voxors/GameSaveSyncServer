#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod add_game_dialog;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui;
use egui::{
    CentralPanel, Context, MenuBar, TopBottomPanel, ViewportBuilder, ViewportCommand, ViewportId,
};

use crate::add_game_dialog::AddGameDialog;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GameSaveSyncClient",
        options,
        Box::new(|_| Ok(Box::<GameSaveClient>::default())),
    )
}

struct GameSaveClient {
    add_game_dialog_opened: Arc<AtomicBool>,
    add_game_dialog_viewport_id: ViewportId,
    add_game_dialog_dialog: Arc<AddGameDialog>,
}

impl Default for GameSaveClient {
    fn default() -> Self {
        let modal_dialog_opened = Arc::new(AtomicBool::new(false));
        Self {
            add_game_dialog_opened: Arc::clone(&modal_dialog_opened),
            add_game_dialog_viewport_id: ViewportId::from_hash_of("add_game_dialog"),
            add_game_dialog_dialog: Arc::new(AddGameDialog::new(
                "Add Game",
                Arc::clone(&modal_dialog_opened),
            )),
        }
    }
}

impl eframe::App for GameSaveClient {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("main_top_panel").show(ctx, |ui| {
            if self.add_game_dialog_opened.load(Ordering::Relaxed) {
                ui.disable();
            }
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    }
                });
                ui.menu_button("Sync", |ui| {
                    if ui.button("Add Game to Sync").clicked() {
                        self.add_game_dialog_opened.store(true, Ordering::SeqCst);
                    }
                })
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            if self.add_game_dialog_opened.load(Ordering::Relaxed) {
                ui.disable();
            }
        });

        if self.add_game_dialog_opened.load(Ordering::Relaxed) {
            let dialog = Arc::clone(&self.add_game_dialog_dialog);
            ctx.show_viewport_deferred(
                self.add_game_dialog_viewport_id,
                ViewportBuilder::default()
                    .with_title("Add Game to Sync")
                    .with_inner_size([320.0, 240.0]),
                move |ctx, _viewport| {
                    dialog.show(ctx);
                },
            );
        }
    }
}
