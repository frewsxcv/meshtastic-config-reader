use meshtastic::Message;
use std::sync::mpsc::{Receiver, Sender, channel};

pub struct App {
    picked_file: Option<Vec<u8>>,
    file_picked_sender: Sender<Vec<u8>>,
    file_picked_receiver: Receiver<Vec<u8>>,
}

impl Default for App {
    fn default() -> Self {
        let (file_picked_sender, file_picked_receiver) = channel();
        Self {
            picked_file: None,
            file_picked_sender,
            file_picked_receiver,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Receive file path from the channel
        if let Ok(path) = self.file_picked_receiver.try_recv() {
            self.picked_file = Some(path);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.separator();
                }

                egui::widgets::global_theme_preference_buttons(ui);

                ui.separator();

                if ui.button("Source code").clicked() {
                    ctx.open_url(egui::OpenUrl::new_tab(
                        "https://github.com/frewsxcv/meshtastic-config-reader",
                    ));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Meshtastic Config Reader");
            ui.label("This tool reads a Meshtastic config file exported from the Android app and prints the contents.");

            if ui.button("Pick file").clicked() {
                // Open the file dialog to pick a file.
                let task = rfd::AsyncFileDialog::new().pick_file();
                let sender = self.file_picked_sender.clone();

                // Await somewhere else
                execute(async move {
                    let file = task.await;

                    if let Some(file) = file {
                        sender.send(file.read().await).unwrap();
                    }
                });
            }

            if let Some(buffer) = &self.picked_file {
                match meshtastic::protobufs::DeviceProfile::decode(buffer.as_slice()) {
                    Ok(device_profile) => {
                        egui::containers::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(
                                egui::widget_text::RichText::new(format!("{device_profile:#?}"))
                                    .monospace(),
                            );
                        });
                    }
                    Err(e) => {
                        ui.label(
                            egui::widget_text::RichText::new(format!("Error: {:#?}", e))
                                .monospace(),
                        );
                    }
                };
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
