use crate::{
    camera::{self, CameraResources},
    renderer::{self, create_render_pipeline, CustomTriangleCallback, RenderResources},
    texture::TextureResource,
};
use egui_wgpu::{self};
use std::sync::{Arc, RwLock};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
///

pub struct TemplateApp {
    // Example stuff:
    viewport_height: f32,
    viewport_width: f32,
    outer_rect: Option<egui::Rect>,
    camera_controller: Arc<RwLock<camera::CameraController>>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap().clone();
        let camera_controller = Arc::new(RwLock::new(camera::CameraController::new(0.2)));

        let (camera_bind_group, camera_bind_group_layout, camera_buffer, camera_uniform) =
            camera::CameraResources::create_camera_bind_group(&wgpu_render_state.device);

        let image_bytes = include_bytes!("happy-tree.png");
        let mut texture_resource = TextureResource::new(&wgpu_render_state, image_bytes).unwrap();

        let texture_bind_group_layout = texture_resource.get_bind_group(&wgpu_render_state);

        let pipeline = Arc::new(create_render_pipeline(
            &wgpu_render_state,
            &[&camera_bind_group_layout, &texture_bind_group_layout],
        ));

        let camera = Arc::new(RwLock::new(camera::Camera::new()));

        let render_state = renderer::Renderer::new(&wgpu_render_state, &pipeline);

        render_state.add_resource(RenderResources::new(&wgpu_render_state, &pipeline));
        render_state.add_resource(texture_resource);
        render_state.add_resource(CameraResources {
            camera_uniform,
            camera_buffer,
            camera: Arc::clone(&camera),
            camera_controller: Arc::clone(&camera_controller),
            camera_bind_group,
            pipeline: Arc::clone(&pipeline),
        });

        Self {
            viewport_height: 800.0,
            viewport_width: 1280.0,
            camera_controller,
            outer_rect: None,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
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

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        ctx.input(|i| self.outer_rect = i.viewport().outer_rect);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.viewport_width, 600.0..=2000.0));
                ui.add(egui::Slider::new(&mut self.viewport_height, 600.0..=2000.0));
            });

            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                let mut style = egui::Style::default();
                style.visuals.extreme_bg_color = egui::Color32::from_rgb(44, 53, 57);
                egui::Frame::canvas(&style).show(ui, |ui| {
                    self.custom_painting(ui);
                });
                ui.label("Drag to rotate!");
            });
        });
    }
}

impl TemplateApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _response) = if let Some(rect) = self.outer_rect.as_ref() {
            ui.allocate_exact_size(
                egui::Vec2::new(rect.width(), rect.height()),
                egui::Sense::drag(),
            )
        } else {
            ui.allocate_exact_size(
                egui::Vec2::new(self.viewport_width, self.viewport_height),
                egui::Sense::drag().union(egui::Sense::click()),
            )
        };

        let mut camera_controller = self.camera_controller.write().unwrap();
        camera_controller.process_events(ui);

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomTriangleCallback {},
        ));
    }
}
