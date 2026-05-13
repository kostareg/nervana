use bevy::{log::info, platform::collections::HashSet, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
use zerocopy::{Immutable, IntoBytes};

const INPUTS_N: usize = 4;
const INTERMEDIATES_N: usize = 10;
const OUTPUTS_N: usize = 3;

const BLOBS_X_N: usize = 10;
const BLOBS_Y_N: usize = 10;
const BLOBS_Z_N: usize = 10;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum Neuron {
    Input(usize),
    Intermediate(usize),
    Output(usize),
}

impl Neuron {
    /// see doc.rust-lang.org/stable/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
    fn get_discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    fn get_value(&self) -> usize {
        match self {
            Self::Input(n) => *n,
            Self::Intermediate(n) => *n,
            Self::Output(n) => *n,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Connection {
    from: Neuron,
    to: Neuron,
    weight: f32,
}

impl Connection {
    fn random() -> Self {
        let inp_rng = || rand::random_range(0..INPUTS_N);
        let int_rng = || rand::random_range(0..INTERMEDIATES_N);
        let out_rng = || rand::random_range(0..OUTPUTS_N);

        let from = *[Neuron::Input(inp_rng()), Neuron::Intermediate(int_rng())].choose(&mut rand::rng()).unwrap();
        let to = *[Neuron::Intermediate(int_rng()), Neuron::Output(out_rng())].choose(&mut rand::rng()).unwrap();

        Self {
            from,
            to,
            weight: rand::random_range(-10. .. 10.),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct NeuralNetwork {
    connections: [Connection; 8],
}

#[derive(Clone, Copy, Debug, Immutable, IntoBytes)]
struct NeuronBytes {
    discriminant: u8,
    _padding: [u8; 7],
    value: usize,
}

impl From<Neuron> for NeuronBytes {
    fn from(value: Neuron) -> Self {
        Self {
            discriminant: value.get_discriminant(),
            _padding: [0; 7],
            value: value.get_value(),
        }
    }
}

#[derive(Clone, Copy, Debug, Immutable, IntoBytes)]
struct ConnectionBytes {
    from: NeuronBytes,
    to: NeuronBytes,
    weight: f32,
    _padding: [u8; 4],
}

impl From<Connection> for ConnectionBytes {
    fn from(value: Connection) -> Self {
        Self {
            from: value.from.into(),
            to: value.to.into(),
            weight: value.weight,
            _padding: [0; 4],
        }
    }
}

impl NeuralNetwork {
    fn random() -> Self {
        Self {
            connections: std::array::from_fn(|_| Connection::random()),
        }
    }

    fn all_bytes(&self) -> Vec<u8> {
        let mut v = Vec::<u8>::with_capacity(64);

        for connection in self.connections {
            let connection_bytes: ConnectionBytes = connection.into();
            let bytes = connection_bytes.as_bytes();
            v.extend_from_slice(bytes);
        }

        v
    }

    fn color(&self) -> (u8, u8, u8) {
        let mut sum_r = 0;
        let mut sum_g = 0;
        let mut sum_b = 0;

        for connection in self.connections {
            let connection_bytes: ConnectionBytes = connection.into();
            let bytes = connection_bytes.as_bytes();

            // we only care about 0, 8, 16, 24, 32-35
            sum_r += (bytes[0] as u16 + bytes[8] as u16 + bytes[16] as u16) / 3;
            sum_g += (bytes[24] as u16 + bytes[32] as u16 + bytes[33] as u16) / 3;
            sum_b += (bytes[34] as u16 + bytes[35] as u16) / 2;
        }

        sum_r /= 8;
        sum_g /= 8;
        sum_b /= 8;

        (sum_r as u8, sum_g as u8, sum_b as u8)
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct Blob {
    network: NeuralNetwork,
    internal_state: [f32; INTERMEDIATES_N],
}

impl Blob {
    fn random() -> Self {
        Self {
            network: NeuralNetwork::random(),
            internal_state: std::array::from_fn(|_| rand::random_range(0. .. 0.5)),
        }
    }

    fn step(&mut self, force: Vec3) -> Vec3 {
        let inputs = [force.x, force.y, force.z, rand::random_range(0. .. f32::MAX)];
        let mut result = force;
        
        for connection in self.network.connections {
            let value = connection.weight * match connection.from {
                Neuron::Input(n) => inputs[n],
                Neuron::Intermediate(n) => self.internal_state[n],
                Neuron::Output(_) => unimplemented!(),
            };

            match connection.to {
                Neuron::Input(_) => unimplemented!(),
                Neuron::Intermediate(n) => self.internal_state[n] = value,
                Neuron::Output(n) => match n {
                    0 => result.x = value / 100.,
                    1 => result.y = value / 100.,
                    2 => result.z = value / 100.,
                    _ => unimplemented!(),
                },
            }
        }

        result.clamp(Vec3 {
            x: -0.0005,
            y: -0.0005,
            z: -0.0005,
        }, Vec3 {
            x: 0.0005,
            y: 0.0005,
            z: 0.0005,
        })
    }
}

fn spawn_environment(
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

fn spawn_blobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in 0..(BLOBS_X_N * BLOBS_Y_N * BLOBS_Z_N) {
        let blob = Blob::random();
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
}

fn step(
    mut query: Query<(&mut Blob, &mut ExternalForce)>
) {
    for (mut blob, mut ext_force) in query.iter_mut() {
        ext_force.force = blob.step(ext_force.force);
    }
}

#[derive(Resource)]
struct CountDown(Timer);

impl Default for CountDown {
    fn default() -> Self {
        Self(Timer::from_seconds(10., TimerMode::Repeating))
    }
}

#[derive(Resource, Default)]
struct Meta {
    survived: usize,
    diversity: usize,
    generation: usize,
}

#[derive(Resource)]
struct SafeZone(Collider, Vec3);

impl SafeZone {
    fn contains_point(&self, point: Vec3) -> bool {
        self.0.contains_point(self.1, Rot::IDENTITY, point)
    }
}

impl Default for SafeZone {
    fn default() -> Self {
        Self(Collider::cuboid(2., 2., 2.), Vec3::new(1., 1., 1.,))
    }
}

fn reset_generation(
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

        for i in 0..(BLOBS_X_N * BLOBS_Y_N * BLOBS_Z_N) {
            let blob = (*samples.choose(&mut rand::rng()).unwrap()).clone();
            let color = blob.network.color();
            seen.insert(blob.network.all_bytes());

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

        meta.survived = counter;
        meta.diversity = seen.len();
        meta.generation += 1;
    }
}

fn ui_example_system(mut contexts: EguiContexts, meta: Res<Meta>, countdown: Res<CountDown>, selected: Res<SelectedBlob>) -> Result {
    egui::Window::new(egui::RichText::new("").size(1.)).show(contexts.ctx_mut()?, |ui| {
        ui.label(egui::RichText::new(format!("Generation: #{}", meta.generation)).size(10.));
        ui.label(egui::RichText::new(format!("Remaining: {:.2}s", countdown.0.remaining_secs())).size(10.));
        ui.label(egui::RichText::new(format!("Survivors: {} blobs ({}%)", meta.survived, 100. * meta.survived as f32 / ((BLOBS_X_N * BLOBS_Y_N * BLOBS_Z_N) as f32))).size(10.));
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

#[derive(Default, Resource)]
struct SelectedBlob(Option<Blob>);

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
