mod blob;
mod network;
mod neurons;
mod simulate;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_rapier3d::prelude::*;

use crate::simulate::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin, EguiPlugin::default(), RapierPhysicsPlugin::<NoUserData>::default()))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<CountDown>()
        .init_resource::<Meta>()
        .init_resource::<SafeZone>()
        .init_resource::<SelectedBlob>()
        .add_systems(Startup, (spawn_environment, spawn_blobs))
        .add_systems(FixedUpdate, (step, reset_generation))
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run();
}
