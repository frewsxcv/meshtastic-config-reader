use std::sync::mpsc::{Receiver, Sender, channel};

pub struct TemplateApp {
    picked_file: Option<Vec<u8>>,
    file_picked_sender: Sender<Vec<u8>>,
    file_picked_receiver: Receiver<Vec<u8>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (file_picked_sender, file_picked_receiver) = channel();
        Self {
            picked_file: None,
            file_picked_sender,
            file_picked_receiver,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
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
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Meshtastic Config Reader");

            if ui.button("Pick file").clicked() {
                // Open the file dialog to pick a file.
                let task = rfd::AsyncFileDialog::new().pick_file();
                let sender = self.file_picked_sender.clone();

                // Await somewhere else
                execute(async move {
                    let file = task.await;

                    if let Some(file) = file {
                        let _ = sender.send(file.read().await);

                        // If you care about wasm support you just read() the file
                        // file.read().await; // We don't need to read the file content for this task
                    }
                });
            }

            ui.label(format!("Picked file: {:?}", self.picked_file));

            /*
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
            */
        });
    }
}

/*
fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
*/

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
