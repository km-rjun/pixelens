#[cfg(feature = "layer-shell")]
use gtk::prelude::*;
#[cfg(feature = "layer-shell")]
use gtk::{self, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation};
#[cfg(feature = "layer-shell")]
use gtk_layer_shell::{Layer, LayerShell};
#[cfg(feature = "layer-shell")]
use std::sync::mpsc;

use crate::error::PixelensError;
use crate::menu::{MenuBackend, MenuChoice};

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

#[cfg(feature = "layer-shell")]
fn build_action_bar(app: &Application, tx: std::sync::mpsc::Sender<MenuChoice>) {
    log::debug!("Building action bar window");

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Pixelens Action Bar")
        .default_width(280)
        .default_height(44)
        .resizable(false)
        .decorated(true)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_interactivity(true);
    window.set_exclusive_zone(-1);

    let content_box = GtkBox::new(Orientation::Horizontal, 8);
    content_box.set_margin_top(4);
    content_box.set_margin_bottom(4);
    content_box.set_margin_start(8);
    content_box.set_margin_end(8);

    let title = Label::new(Some("Action:"));
    content_box.pack_start(&title, false, false, 0);

    let actions = [
        ("[C] Copy", MenuChoice::Copy),
        ("[S] Search", MenuChoice::Search),
        ("[A] Ask AI", MenuChoice::Ai),
        ("[T] Translate", MenuChoice::Translate),
        ("[Esc] Cancel", MenuChoice::Cancel),
    ];

    for (label_text, choice) in actions {
        let button = Button::builder().label(label_text).build();
        let tx_clone = tx.clone();
        let app_clone = app.clone();
        button.connect_clicked(move |_| {
            log::debug!("Button clicked: {:?}", choice);
            let _ = tx_clone.send(choice.clone());
            app_clone.quit();
        });
        content_box.pack_start(&button, false, false, 0);
    }

    window.add(&content_box);
    window.show_all();
    log::debug!("Action bar window presented");
}

pub fn show_action_bar() -> Result<MenuChoice, PixelensError> {
    #[cfg(feature = "layer-shell")]
    {
        let (tx, rx) = mpsc::channel();

        log::debug!("Creating GTK application");
        let app = Application::builder()
            .application_id("com.pixelens.action-bar")
            .build();

        let tx_clone = tx.clone();
        app.connect_activate(move |app| {
            log::debug!("Activate callback fired");
            build_action_bar(app, tx_clone.clone());
        });

        log::debug!("Running GTK application");
        app.run();

        log::debug!("GTK application exited, receiving from channel");
        rx.recv()
            .map_err(|e| PixelensError::Config(format!("Menu channel closed: {}", e)))
    }

    #[cfg(not(feature = "layer-shell"))]
    {
        Err(PixelensError::Config(
            "Pixelens was built without the Cargo feature `layer-shell`. \
             Rebuild with: cargo build --release --workspace --features layer-shell"
                .to_string(),
        ))
    }
}

pub fn create_backend() -> Result<Box<dyn MenuBackend>, PixelensError> {
    #[cfg(feature = "layer-shell")]
    {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let app = Application::builder()
                .application_id("com.pixelens.action-bar")
                .build();

            let tx_clone = tx.clone();
            app.connect_activate(move |app| {
                build_action_bar(app, tx_clone.clone());
            });

            app.run();
        });

        Ok(Box::new(ActionBarBackend { rx }))
    }

    #[cfg(not(feature = "layer-shell"))]
    {
        Err(PixelensError::Config(
            "Pixelens was built without the Cargo feature `layer-shell`. \
             Rebuild with: cargo build --release --workspace --features layer-shell"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_bar_name() {
        let backend = ActionBarBackend {
            rx: std::sync::mpsc::channel().1,
        };
        assert_eq!(backend.name(), "action_bar");
    }

    #[test]
    fn test_menu_choice_dispatch() {
        let (tx, rx) = std::sync::mpsc::channel();

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
