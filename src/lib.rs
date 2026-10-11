mod protocol;

use leafwing_input_manager::action_state::ActionState;
use lightyear::avian3d::plugin::*;

use lightyear::prediction::Predicted;
pub use protocol::*;
pub use std::net::*;
pub use std::time::*;

pub const TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_PORT: u16 = 5000;

pub const CLIENT_PORT: u16 = 4000;
pub const SHARED_SETTINGS: SharedSettings = SharedSettings {
    protocol_id: 1,
    private_key: [0; 32],
};

const MAX_SPEED: f32 = 12.0;
const MAX_ACCELERATION: f32 = 8.0;
const SHIFT_MULTIPLIER: f32 = 1.7;

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

        app.add_systems(FixedUpdate, (player_rotation, player_movement));
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

pub fn player_movement(
    time: Res<Time>,
    mut query: Query<(
        &ComputedMass,
        &ActionState<PlayerAction>,
        Forces,
        Has<Predicted>,
    )>,
) {
    for (mass, action, mut forces, _predicted) in query.iter_mut() {
        #[cfg(feature = "client")]
        if !_predicted {
            continue;
        }

        let velocity_delta = MAX_ACCELERATION * time.delta_secs();
        let m = action.axis_pair(&PlayerAction::Move).normalize_or_zero();

        if m == Vec2::ZERO {
            continue;
        }

        let move_dir = Vec3::new(m.x, 0.0, m.y);

        let linear_velocity = forces.linear_velocity();
        let ground_linear = Vec3::new(linear_velocity.x, 0.0, linear_velocity.z);
        let mut desired_linear = move_dir * MAX_SPEED;
        if action.pressed(&PlayerAction::Shift) {
            desired_linear *= SHIFT_MULTIPLIER;
        }

        let new_ground_velocity = ground_linear.move_towards(desired_linear, velocity_delta);
        let acceleration = (new_ground_velocity - ground_linear) / time.delta_secs();

        forces.apply_force(acceleration * mass.value());
    }
}
