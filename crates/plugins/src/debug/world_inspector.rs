use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContext, PrimaryEguiContext, egui};
use bevy_inspector_egui::bevy_inspector::ui_for_world;
use bevy_inspector_egui::egui::Align2;

pub fn world_inspector_ui(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("World Inspector")
        .default_size((320., 160.))
        .anchor(Align2::RIGHT_TOP, [-5., 5.])
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}
