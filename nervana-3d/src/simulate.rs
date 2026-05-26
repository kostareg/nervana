use crate::blob::*;

use bevy::{log::info, platform::collections::HashSet, prelude::*};
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use rand::prelude::*;

/// Blobs spawned along the x axis of the starting grid.
const BLOBS_X_N: usize = 10;
/// Blobs spawned along the y axis of the starting grid.
const BLOBS_Y_N: usize = 10;
/// Blobs spawned along the z axis of the starting grid.
const BLOBS_Z_N: usize = 10;

/// Total number of blobs spawned per generation.
const BLOBS_N: usize = BLOBS_X_N * BLOBS_Y_N * BLOBS_Z_N;

/// Time left in the current generation; when it elapses the population is
/// culled and respawned.
#[derive(Resource)]
pub struct CountDown(Timer);

impl Default for CountDown {
    fn default() -> Self {
        Self(Timer::from_seconds(10., TimerMode::Repeating))
    }
}

/// Per-generation statistics surfaced in the inspector UI.
#[derive(Resource, Default)]
pub struct Meta {
    /// Blobs that were inside the [`SafeZone`] when the generation ended.
    survived: usize,
    /// Number of distinct networks among the respawned population.
    diversity: usize,
    /// Current generation number.
    generation: usize,
}

/// The region a blob must be in to survive a generation. Holds the survival
/// volume and its center position.
#[derive(Resource)]
pub struct SafeZone(Collider, Vec3);

impl SafeZone {
    /// Returns whether `point` lies inside the safe zone.
    fn contains_point(&self, point: Vec3) -> bool {
        self.0.contains_point(self.1, Rot::IDENTITY, point)
    }
}

impl Default for SafeZone {
    fn default() -> Self {
        Self(Collider::cuboid(2., 2., 2.), Vec3::new(1., 1., 1.,))
    }
}

/// The blob currently hovered by the cursor, shown in the inspector UI.
#[derive(Default, Resource)]
pub struct SelectedBlob(Option<Blob>);

/// Spawn a single blob entity at grid index `i`, wiring up the hover observers
/// that highlight it and feed the inspector UI.
fn spawn_blob(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    blob: Blob,
    i: usize,
) {
    let color = blob.network.color();

    commands.spawn((
        blob,
        Collider::cuboid(0.06, 0.06, 0.06),
        RigidBody::Dynamic,
        GravityScale(0.),
        ExternalForce { force: Vec3::ZERO, torque: Vec3::ZERO },
        Restitution::coefficient(0.7),
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
        MeshMaterial3d(materials.add(Color::srgb_u8(color.0, color.1, color.2))),
        Transform::from_xyz((i % BLOBS_X_N) as f32 / 2. - 2.5, ((i / (BLOBS_X_N * BLOBS_Z_N)) as f32) / 2. - 2.5, (((i / BLOBS_X_N) as f32) % (BLOBS_Z_N as f32)) / 2. - 2.5),
    )).observe(|trigger: Trigger<Pointer<Over>>, query: Query<(&Blob, &MeshMaterial3d<StandardMaterial>)>, mut selected: ResMut<SelectedBlob>, mut materials: ResMut<Assets<StandardMaterial>>| {
        let (blob, handle) = query.get(trigger.target()).unwrap();
        selected.0 = Some(*blob);
        materials.get_mut(handle).unwrap().base_color = Color::srgb_u8(255, 0, 0);
    }).observe(|trigger: Trigger<Pointer<Out>>, query: Query<(&Blob, &MeshMaterial3d<StandardMaterial>)>, mut materials: ResMut<Assets<StandardMaterial>>| {
        let (blob, handle) = query.get(trigger.target()).unwrap();
        let color = blob.network.color();
        materials.get_mut(handle).unwrap().base_color = Color::srgb_u8(color.0, color.1, color.2);
    });
}

/// Startup system that builds the static scene: the bounding box the blobs are
/// trapped in, the safe-zone sensor, the floor, a light, and the camera.
pub fn spawn_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    safe_zone: Res<SafeZone>,
) {
    let hx = 3.;
    let hy = 3.;
    let hz = 3.;

    let verts = vec![
        Vec3::new(-hx, -hy, -hz),
        Vec3::new( hx, -hy, -hz),
        Vec3::new( hx,  hy, -hz),
        Vec3::new(-hx,  hy, -hz),
        Vec3::new(-hx, -hy,  hz),
        Vec3::new( hx, -hy,  hz),
        Vec3::new( hx,  hy,  hz),
        Vec3::new(-hx,  hy,  hz),
    ];

    let indices = vec![
        [0,1,2], [0,2,3],
        [4,6,5], [4,7,6],
        [0,4,5], [0,5,1],
        [3,2,6], [3,6,7],
        [1,5,6], [1,6,2],
        [0,3,7], [0,7,4],
    ];

    commands.spawn((
        RigidBody::Fixed,
        Collider::trimesh(verts, indices).unwrap(),
    ));

    commands.spawn((
        safe_zone.0.clone(),
        Transform::from_translation(safe_zone.1),
        Sensor,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(3.)))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
        Transform::from_xyz(0., -3., 0.),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 4.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-9., 9., 9.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Startup system that fills the world with the initial generation of random
/// blobs.
pub fn spawn_blobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in 0..BLOBS_N {
        spawn_blob(&mut commands, &mut meshes, &mut materials, Blob::random(), i);
    }
}

/// Fixed-update system that ticks every blob's network and applies the force it
/// produces.
pub fn step(
    mut query: Query<(&mut Blob, &mut ExternalForce)>
) {
    for (mut blob, mut ext_force) in query.iter_mut() {
        ext_force.force = blob.step(ext_force.force);
    }
}

/// Fixed-update system that runs the evolutionary loop: when the generation
/// timer fires, it keeps the blobs inside the safe zone, repopulates the world
/// from their networks, and records the generation's stats.
pub fn reset_generation(
    time: Res<Time>,
    safe_zone: Res<SafeZone>,
    mut countdown: ResMut<CountDown>,
    mut meta: ResMut<Meta>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Blob, &Transform), With<Blob>>,
) {
    if countdown.0.tick(time.delta()).just_finished() {
        info!("Resetting generation");

        let mut counter = 0;
        let mut seen = HashSet::<Vec<u8>>::new();
        let mut samples = Vec::<&Blob>::new();

        for (entity, blob, transform) in query {
            if safe_zone.contains_point(transform.translation) {
                counter += 1;
                samples.push(blob);
            }
            commands.entity(entity).despawn();
        }

        for i in 0..BLOBS_N {
            let blob = (*samples.choose(&mut rand::rng()).unwrap()).clone();
            seen.insert(blob.network.all_bytes());

            spawn_blob(&mut commands, &mut meshes, &mut materials, blob, i);
        }

        meta.survived = counter;
        meta.diversity = seen.len();
        meta.generation += 1;
    }
}

/// Draws the egui overlay showing generation stats and details of the blob
/// currently under the cursor.
pub fn ui_example_system(mut contexts: EguiContexts, meta: Res<Meta>, countdown: Res<CountDown>, selected: Res<SelectedBlob>) -> Result {
    egui::Window::new(egui::RichText::new("").size(1.)).show(contexts.ctx_mut()?, |ui| {
        ui.label(egui::RichText::new(format!("Generation: #{}", meta.generation)).size(10.));
        ui.label(egui::RichText::new(format!("Remaining: {:.2}s", countdown.0.remaining_secs())).size(10.));
        ui.label(egui::RichText::new(format!("Survivors: {} blobs ({}%)", meta.survived, 100. * meta.survived as f32 / (BLOBS_N as f32))).size(10.));
        ui.label(egui::RichText::new(format!("Diversity: {} variants", meta.diversity)).size(10.));
        // todo: more detail about the selected neural network here
        if let Some(blob) = selected.0 {
            let color = blob.network.color();
            ui.separator();
            ui.label(egui::RichText::new(format!("Selected blob color: {} {} {}", color.0, color.1, color.2)).size(10.));
        }
    });
    Ok(())
}
