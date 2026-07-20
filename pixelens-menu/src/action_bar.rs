//! Graphical action-bar backend built on `gtk4` + `gtk-layer-shell`.
//!
//! Presents a small overlay window with one button per action and reports the
//! chosen [`MenuChoice`] back over an `mpsc` channel. The GTK machinery is
//! compiled only when the `menu-gtk` feature is enabled; without it this
//! module still exposes [`ActionBarBackend`] (so `name()` is testable) and
//! [`create_backend`] returns a clear "feature disabled" error.

#[cfg(feature = "menu-gtk")]
use gtk::prelude::*;
#[cfg(feature = "menu-gtk")]
use gtk::{self, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation};
#[cfg(feature = "menu-gtk")]
use gtk_layer_shell::{Layer, LayerShell};
#[cfg(feature = "menu-gtk")]
use std::sync::mpsc;

use crate::types::{MenuBackend, MenuChoice, MenuError};

/// Receiver-backed [`MenuBackend`]. The actual GUI lives on a spawned GTK
/// thread (see [`create_backend`]); this half just consumes the choice.
pub struct ActionBarBackend {
    #[cfg(feature = "menu-gtk")]
    rx: mpsc::Receiver<MenuChoice>,
}

impl MenuBackend for ActionBarBackend {
    #[cfg(feature = "menu-gtk")]
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, MenuError> {
        self.rx
            .recv()
            .map_err(|e| MenuError::ChannelClosed(format!("action bar channel closed: {e}")))
    }

    #[cfg(not(feature = "menu-gtk"))]
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, MenuError> {
        Err(MenuError::Backend(
            "menu-gtk feature disabled; action bar unavailable".into(),
        ))
    }

    fn name(&self) -> &str {
        "action_bar"
    }
}

#[cfg(feature = "menu-gtk")]
fn build_action_bar(app: &Application, tx: mpsc::Sender<MenuChoice>) {
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
            let _ = tx_clone.send(choice.clone());
            app_clone.quit();
        });
        content_box.pack_start(&button, false, false, 0);
    }

    window.add(&content_box);
    window.show_all();
}

/// Build the action-bar backend, spawning the GTK event loop on a background
/// thread. Returns a receiver-backed [`ActionBarBackend`].
#[cfg(feature = "menu-gtk")]
pub fn create_backend() -> Result<Box<dyn MenuBackend + Send + Sync + 'static>, MenuError> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let app = Application::builder()
            .application_id("com.pixelens.action-bar")
            .build();

        let tx_clone = tx.clone();
        app.connect_activate(move |app| {
            build_action_bar(app, tx_clone.clone());
        });

        let synthetic_args = ["pixelens-action-bar"];
        app.run_with_args(&synthetic_args);
    });

    Ok(Box::new(ActionBarBackend { rx }))
}

/// Without the `menu-gtk` feature there is no GTK to spawn.
#[cfg(not(feature = "menu-gtk"))]
pub fn create_backend() -> Result<Box<dyn MenuBackend + Send + Sync + 'static>, MenuError> {
    Err(MenuError::Backend(
        "Pixelens was built without the Cargo feature `menu-gtk`. \
         Rebuild with: cargo build --release --features menu-gtk"
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_action_bar() {
        let backend = ActionBarBackend {
            #[cfg(feature = "menu-gtk")]
            rx: mpsc::channel().1,
        };
        assert_eq!(backend.name(), "action_bar");
    }
}
