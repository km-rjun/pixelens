use eframe::egui;
use std::sync::mpsc;

use crate::error::PixelensError;
use crate::menu::{MenuBackend, MenuChoice};

struct ActionBarApp {
    tx: mpsc::Sender<MenuChoice>,
}

impl eframe::App for ActionBarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.heading("Action");
                ui.separator();

                if ui.button("[C] Copy").clicked() {
                    let _ = self.tx.send(MenuChoice::Copy);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if ui.button("[S] Search").clicked() {
                    let _ = self.tx.send(MenuChoice::Search);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if ui.button("[A] Ask AI").clicked() {
                    let _ = self.tx.send(MenuChoice::Ai);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if ui.button("[T] Translate").clicked() {
                    let _ = self.tx.send(MenuChoice::Translate);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if ui.button("[Esc] Cancel").clicked() {
                    let _ = self.tx.send(MenuChoice::Cancel);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            let _ = self.tx.send(MenuChoice::Cancel);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        for key in [
            (egui::Key::C, MenuChoice::Copy),
            (egui::Key::S, MenuChoice::Search),
            (egui::Key::A, MenuChoice::Ai),
            (egui::Key::T, MenuChoice::Translate),
        ] {
            if ctx.input(|i| i.key_pressed(key.0)) {
                let _ = self.tx.send(key.1);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

pub struct ActionBar;

pub struct ActionBarBackend {
    rx: std::sync::mpsc::Receiver<MenuChoice>,
}

impl MenuBackend for ActionBarBackend {
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, PixelensError> {
        self.rx
            .recv()
            .map_err(|e| PixelensError::Config(format!("Menu channel closed: {}", e)))
    }

    fn name(&self) -> &str {
        "action_bar"
    }
}

pub fn show_action_bar() -> Result<MenuChoice, PixelensError> {
    let (tx, rx) = mpsc::channel();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([250.0, 40.0])
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };

    let app = ActionBarApp { tx };

    let result = eframe::run_native("Action Bar", options, Box::new(|_cc| Ok(Box::new(app))));

    match result {
        Ok(()) => {}
        Err(e) => {
            log::warn!("Action bar window closed: {}", e);
        }
    }

    rx.recv()
        .map_err(|e| PixelensError::Config(format!("Menu channel closed: {}", e)))
}

pub fn create_backend() -> Result<Box<dyn MenuBackend>, PixelensError> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([250.0, 40.0])
                .with_decorations(false)
                .with_transparent(true),
            ..Default::default()
        };
        let app = ActionBarApp { tx };
        let _ = eframe::run_native("Action Bar", options, Box::new(|_cc| Ok(Box::new(app))));
    });

    Ok(Box::new(ActionBarBackend { rx }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_bar_name() {
        let backend = ActionBarBackend {
            rx: mpsc::channel().1,
        };
        assert_eq!(backend.name(), "action_bar");
    }

    #[test]
    fn test_menu_choice_dispatch() {
        let (tx, rx) = mpsc::channel();

        tx.send(MenuChoice::Copy).unwrap();
        assert_eq!(rx.recv().unwrap(), MenuChoice::Copy);

        tx.send(MenuChoice::Search).unwrap();
        assert_eq!(rx.recv().unwrap(), MenuChoice::Search);

        tx.send(MenuChoice::Ai).unwrap();
        assert_eq!(rx.recv().unwrap(), MenuChoice::Ai);

        tx.send(MenuChoice::Translate).unwrap();
        assert_eq!(rx.recv().unwrap(), MenuChoice::Translate);

        tx.send(MenuChoice::Cancel).unwrap();
        assert_eq!(rx.recv().unwrap(), MenuChoice::Cancel);
    }

    #[test]
    fn test_menu_choice_from_key() {
        assert_eq!(MenuChoice::from_key("c"), Some(MenuChoice::Copy));
        assert_eq!(MenuChoice::from_key("s"), Some(MenuChoice::Search));
        assert_eq!(MenuChoice::from_key("a"), Some(MenuChoice::Ai));
        assert_eq!(MenuChoice::from_key("t"), Some(MenuChoice::Translate));
        assert_eq!(MenuChoice::from_key("escape"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("q"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("x"), None);
    }
}
