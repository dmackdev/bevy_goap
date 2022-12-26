use bevy::prelude::Plugin;
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};

use crate::{action::Action, actor::ActorState, planning::PlanningState, ActionState, Actor};

pub struct GoapInspectorPlugin;

impl Plugin for GoapInspectorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .register_inspectable::<Actor>()
            .register_inspectable::<ActorState>()
            .register_inspectable::<Action>()
            .register_inspectable::<ActionState>()
            .register_inspectable::<PlanningState>();
    }
}

impl Inspectable for Actor {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _options: Self::Attributes,
        _context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        ui.label(format!("{:#?}", self));
        false
    }
}

impl Inspectable for ActorState {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _options: Self::Attributes,
        _context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        ui.label(format!("{:#?}", self));
        false
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
        ui.label(format!("{:#?}", self));
        false
    }
}

impl Inspectable for ActionState {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _options: Self::Attributes,
        _context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        ui.label(format!("{:#?}", self));
        false
    }
}

impl Inspectable for PlanningState {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _options: Self::Attributes,
        _context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        ui.label(format!("{:#?}", self));
        false
    }
}
