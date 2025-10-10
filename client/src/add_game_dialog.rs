use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui::{CentralPanel, Context};
use egui::ViewportCommand;

#[derive(Clone)]
pub struct AddGameDialog {
    pub title: String,
    opened: Arc<AtomicBool>,
}

impl AddGameDialog {
    pub fn new(title: impl Into<String>, opened: Arc<AtomicBool>) -> Self {
        Self {
            title: title.into(),
            opened,
        }
    }

    pub fn show(&self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.title);
            if ui.button("Save").clicked() {
                self.opened.store(false, Ordering::SeqCst);
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            if ui.button("Close").clicked() {
                self.opened.store(false, Ordering::SeqCst);
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            if ctx.input(|i| i.viewport().close_requested()) {
                self.opened.store(false, Ordering::Relaxed);
            }
        });
    }
}
