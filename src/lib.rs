pub mod config;
pub mod protocol;
pub mod scene;
pub mod servers;
pub mod status;

use leafwing_input_manager::action_state::ActionState;
use lightyear::avian3d::plugin::*;

use lightyear::prediction::Predicted;
pub use protocol::*;
pub use std::net::*;
pub use std::time::*;

pub const TIMESTEP_HZ: f64 = 64.0;

pub const MAX_PLAYERS: u8 = 8;

pub const CLIENT_PORT: u16 = 4000;

const PLAYER_SPEED: f32 = 3.0;
const SHIFT_MULTIPLIER: f32 = 1.17;

#[derive(Copy, Clone, Debug)]
pub struct SharedSettings {
    pub protocol_id: u64,
    pub private_key: [u8; 32],
}

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_plugins(lightyear::avian3d::plugin::LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position {
                sync_to_transform: false,
            },
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

        app.add_systems(FixedUpdate, (player_rotation, player_movement).chain());
    }
}

pub fn player_rotation(
    mut query: Query<(Mut<Rotation>, &ActionState<PlayerAction>, Has<Predicted>)>,
) {
    for (mut rotation, action, _predicted) in query.iter_mut() {
        #[cfg(feature = "client")]
        if !_predicted {
            continue;
        }

        let r = action.axis_pair(&PlayerAction::Rotation);
        rotation.0 = Quat::from_rotation_y(r.y);
    }
}

pub fn player_movement(mut query: Query<(&ActionState<PlayerAction>, Forces, Has<Predicted>)>) {
    for (action, mut forces, _predicted) in query.iter_mut() {
        #[cfg(feature = "client")]
        if !_predicted {
            continue;
        }

        let m = action.axis_pair(&PlayerAction::Move).normalize_or_zero();
        if m == Vec2::ZERO {
            continue;
        }

        let rotation = forces.rotation();
        let forward = -(rotation * Vec3::Z) * m.y;
        let right = rotation * Vec3::X * m.x;
        let move_dir = forward + right;

        let f = (rotation * Vec3::NEG_Z).xz();
        let alpha = f.angle_to(move_dir.xz());
        let l = 0.8 + 0.2 * alpha.cos();

        let mut velocity = move_dir * l * PLAYER_SPEED;
        if action.pressed(&PlayerAction::Shift) {
            velocity *= SHIFT_MULTIPLIER;
        }

        forces.linear_velocity_mut().x = velocity.x;
        forces.linear_velocity_mut().z = velocity.z;
    }
}
