use bevy::prelude::Plugin;
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};

use crate::Action;

pub struct GoapInspectorPlugin;

impl Plugin for GoapInspectorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .register_inspectable::<Action>();
    }
}

impl Inspectable for Action {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _options: Self::Attributes,
        _context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        ui.label(format!("preconditions: {:#?}", self.preconditions));
        false
    }
}
