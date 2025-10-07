use std::f32::consts::PI;

use bevy::{
    ecs::relationship::RelationshipSourceCollection,
    light::CascadeShadowConfigBuilder,
    prelude::*,    
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
    window::PrimaryWindow,
};

use bevy_egui::{
    EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass,
    PrimaryEguiContext, egui,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};



pub fn styled_button(ui: &mut egui::Ui, text: &str, dir: bool, is_selected: bool) -> egui::Response {
    // Define colors for different states
    let (bg_color, hover_color, text_color) = match (dir, is_selected) {
        // Selected file - blue theme
        (false, true) => (
            egui::Color32::from_rgb(80, 80, 90),
            egui::Color32::from_rgb(70, 130, 21),
            egui::Color32::WHITE,
        ),
        // Regular file -
        (false, false) => (
            egui::Color32::from_rgb(28, 29, 30),
            egui::Color32::from_rgb(20, 20, 20),
            egui::Color32::from_rgb(200, 202, 203),
        ),
        // Directory - 
        (true, _) => (
            egui::Color32::from_rgb(30, 32, 44),
            egui::Color32::from_rgb(25, 20, 10),
            egui::Color32::from_rgb(200, 202, 203),
        ),
    };

    // Create custom button style
    let button = egui::Button::new(egui::RichText::new(format!("{} ", text)).color(text_color))
        .fill(bg_color)
        .stroke(egui::Stroke::new(
            1.0,
            if is_selected {
                egui::Color32::from_rgba_premultiplied(120, 120, 120, 64)
            } else {
                egui::Color32::from_rgb(24, 24, 24)
            },
        ))
        .rounding(4.0)
        .min_size(egui::vec2(200.0, 30.0));

    // Apply hover color
    let response = ui.add(button);

    // Custom hover effect using style
    if response.hovered() {
        // ui.painter().rect_filled(response.rect, 4.0, hover_color);
    }

    response
}