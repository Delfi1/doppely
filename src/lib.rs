//! Сервер управялет ИИ агентами, занимается их обучением, а так-же работает с подключениями игроков.
//! Для создания нейросети используется крейт [dfdx](https://docs.rs/dfdx/latest/dfdx/)
//! Для игровой логики используется движок [bevy](https://docs.rs/bevy/latest/bevy/)
//! Для клиент-серверного взаимодействия используется [lightyear](https://docs.rs/lightyear/latest/lightyear/)

mod protocol;
use leafwing_input_manager::prelude::*;
use lightyear::avian3d::plugin::*;
pub use protocol::*;

use bevy::prelude::*;
pub use core::net::*;

pub const EPS: f32 = 0.0001;
pub const MAX_VELOCITY: f32 = 20.0;
pub const SENSITIVITY: f32 = 0.08;
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

pub const TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_PORT: u16 = 5000;
pub const PLAYER_SPEED: f32 = 5.0;
pub const MAX_ACCELERATION: f32 = 20.0;
pub const SHIFT_MULTIPLIER: f32 = 1.7;

pub const CHARACTER_WIDTH: f32 = 0.5;
pub const CHARACTER_HEIGHT: f32 = 1.2;

pub fn shared_movement(
    action: &ActionState<PlayerAction>,
    time: &Time,
    //mass: &ComputedMass,
    mut transform: Mut<Transform>,
    //mut forces: ForcesItem,
) {
    // rotate player:
    if let Some(cursor_data) = action.dual_axis_data(&PlayerAction::MouseMove) {
        let delta = cursor_data.pair * time.delta_secs();
        let (mut yaw, mut pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);

        yaw -= delta.x * SENSITIVITY;
        pitch -= delta.y * SENSITIVITY;
        pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }

    let Some(move_data) = action.dual_axis_data(&PlayerAction::Move) else {
        return;
    };

    // move:
    let mut movement = Vec3::ZERO;
    movement += transform.forward() * move_data.pair.y;
    movement += transform.right() * move_data.pair.x;

    // remove vertical movement
    movement.y = 0.0;

    if let Some(normalized) = movement.try_normalize() {
        movement = normalized * PLAYER_SPEED * time.delta_secs();
        if action.pressed(&PlayerAction::Shift) {
            movement *= SHIFT_MULTIPLIER;
        }

        transform.translation += movement;
    }
}

pub fn character_physics() -> impl Bundle {
    (
        Collider::capsule(CHARACTER_WIDTH, CHARACTER_HEIGHT),
        RigidBody::Dynamic,
        LockedAxes::default()
            .lock_rotation_x()
            .lock_rotation_y()
            .lock_rotation_z(),
        Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
    )
}

pub fn dynamic_physics() -> impl Bundle {
    (Collider::cuboid(1.0, 1.0, 1.0), RigidBody::Dynamic)
}

pub fn static_physics() -> impl Bundle {
    (Collider::cuboid(1.0, 1.0, 1.0), RigidBody::Static)
}

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_plugins(lightyear::avian3d::plugin::LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Transform,
            ..default()
        });

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        );
    }
}
