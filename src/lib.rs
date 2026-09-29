//! Сервер управялет ИИ агентами, занимается их обучением, а так-же работает с подключениями игроков.
//! Для создания нейросети используется крейт [dfdx](https://docs.rs/dfdx/latest/dfdx/)
//! Для игровой логики используется движок [bevy](https://docs.rs/bevy/latest/bevy/)
//! Для клиент-серверного взаимодействия используется [lightyear](https://docs.rs/lightyear/latest/lightyear/)

mod protocol;
use leafwing_input_manager::prelude::*;
use lightyear::{prediction::Predicted, prelude::SyncedLocalTimeline};
pub use protocol::*;

use bevy::prelude::*;
pub use core::net::*;

pub const EPS: f32 = 0.0001;
pub const SENSITIVITY: f32 = 0.08;
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

pub const TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_PORT: u16 = 5000;
pub const PLAYER_SPEED: f32 = 5.0;
pub const SHIFT_MULTIPLIER: f32 = 1.7;

pub fn shared_movement(
    action: &ActionState<PlayerAction>,
    time: &Time,
    mut transform: Mut<Transform>,
) {
    // rotate player:
    let Some(cursor_data) = action.dual_axis_data(&PlayerAction::MoveCursor) else {
        return;
    };

    let delta = cursor_data.pair;
    let (mut yaw, mut pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);

    yaw -= delta.x * SENSITIVITY;
    pitch -= delta.y * SENSITIVITY;
    pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

    // move:
    let mut movement = Vec3::ZERO;
    let forward = *transform.forward();
    let right = *transform.right();
    if action.pressed(&PlayerAction::Forward) {
        movement += forward;
    }
    if action.pressed(&PlayerAction::Backward) {
        movement -= forward;
    }
    if action.pressed(&PlayerAction::Left) {
        movement -= right;
    }
    if action.pressed(&PlayerAction::Right) {
        movement += right;
    }

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

fn player_movement(
    time: Res<Time>,
    input_timeline: Option<SyncedLocalTimeline>,
    mut player_query: Query<
        (Mut<Transform>, Has<Predicted>, &ActionState<PlayerAction>),
        With<PlayerId>,
    >,
) {
    let client_is_synced = input_timeline.is_some();
    for (transform, is_predicted, action_state) in player_query.iter_mut() {
        if is_predicted && !client_is_synced {
            continue;
        }
        shared_movement(action_state, &time, transform);
    }
}

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        // app.add_plugins(
        //     PhysicsPlugins::default()
        //         .build()
        //         .disable::<PhysicsTransformPlugin>(),
        // )
        // .insert_resource(Gravity(-Vec3::Y));
        app.add_plugins(lightyear::avian3d::plugin::LightyearAvianPlugin::default());

        app.add_systems(FixedUpdate, player_movement);
    }
}
