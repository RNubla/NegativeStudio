use egui_extras::RetainedImage;
use image::{imageops, GenericImageView, ImageBuffer, RgbImage};
use poll_promise::Promise;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    url: String,
    #[serde(skip)] // don't serialize this
    promise: Option<Promise<ehttp::Result<RetainedImage>>>,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            url: "".to_owned(),
            promise: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            url,
            promise: _,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Image").clicked() {
                        println!("Open Image");
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            // if ui.button("Load Image").clicked(){}

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });
        // egui::CentralPanel::default().show(ctx, |ui| match promise.ready() {
        //     None => {
        //         ui.spinner(); // still loading
        //     }
        //     Some(Err(err)) => {
        //         ui.colored_label(egui::Color32::RED, err); // something went wrong
        //     }
        //     Some(Ok(image)) => {
        //         image.show_max_size(ui, ui.available_size());
        //         egui::warn_if_debug_build(ui);
        //     }
        // });
        egui::CentralPanel::default().show(ctx, |ui| {
            let trigger_fetch = ui_url(ui, url);

            if trigger_fetch {
                let ctx = ctx.clone();
                let (sender, promise) = Promise::new();
                let request = ehttp::Request::get(url);
                ehttp::fetch(request, move |response| {
                    let image = response.and_then(parse_response);
                    // let img = image::open(image).expect("Failed to load image");
                    sender.send(image); // send the results back to the UI thread.
                    ctx.request_repaint(); // wake up UI thread
                });
                self.promise = Some(promise);
            }

            if let Some(promise) = &self.promise {
                match promise.ready() {
                    None => {
                        ui.spinner(); // still loading
                    }
                    Some(Err(err)) => {
                        ui.colored_label(egui::Color32::RED, err); // something went wrong
                    }
                    Some(Ok(image)) => {
                        image.show_max_size(ui, ui.available_size());
                        egui::warn_if_debug_build(ui);
                    }
                }
            }

            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

fn ui_url(ui: &mut egui::Ui, url: &mut String) -> bool {
    let mut trigger_fetch = false;
    ui.horizontal(|ui| {
        ui.label("URL:");
        trigger_fetch |= ui
            .add(egui::TextEdit::singleline(url).desired_width(f32::INFINITY))
            .lost_focus();
    });
    trigger_fetch
}

// fn ui_url

#[allow(clippy::needless_pass_by_value)]
fn parse_response(response: ehttp::Response) -> Result<RetainedImage, String> {
    let content_type = response.content_type().unwrap_or_default();
    if content_type.starts_with("image/") {
        RetainedImage::from_image_bytes(&response.url, &response.bytes)
    } else {
        Err(format!(
            "Expected image, found content-type {:?}",
            content_type
        ))
    }
}
