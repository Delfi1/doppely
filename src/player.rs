use crate::PlayerAction;
use avian3d::{dynamics::rigid_body::forces::ForcesItem, prelude::*};
use bevy::prelude::*;
use leafwing_input_manager::action_state::*;

pub const SENSITIVITY: f32 = 0.08;
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.001;

const MAX_SPEED: f32 = 20.0;
const MAX_ACCELERATION: f32 = 20.0;
const SHIFT_MULTIPLIER: f32 = 1.7;

pub fn shared_rotation(
    action: &ActionState<PlayerAction>,
    time: &Res<Time>,
    rotation: &mut Rotation,
) {
    if let Some(data) = action.dual_axis_data(&PlayerAction::MouseMove) {
        let delta = data.pair * time.delta_secs();
        let (mut yaw, mut pitch, roll) = rotation.to_euler(EulerRot::YXZ);

        yaw -= delta.x * SENSITIVITY;
        pitch -= delta.y * SENSITIVITY;
        pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

        rotation.0 = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

pub fn shared_movement(
    action: &ActionState<PlayerAction>,
    time: &Res<Time>,
    mass: &ComputedMass,
    mut forces: ForcesItem,
) {
    let data = action.axis_pair(&PlayerAction::Move).clamp_length_max(1.0);

    let rotation = forces.rotation();
    let move_dir = (*Dir3::new_unchecked(rotation.0 * Vec3::X) * data.x
        - *Dir3::new_unchecked(rotation.0 * Vec3::Z) * data.y)
        .with_y(0.0);

    let max_velocity = MAX_ACCELERATION * time.delta_secs();
    let ground_linear = forces.linear_velocity().with_y(0.0);
    let mut desired_velocity = move_dir * MAX_SPEED;
    if action.pressed(&PlayerAction::Shift) {
        desired_velocity *= SHIFT_MULTIPLIER;
    }

    let result_velocity = ground_linear.move_towards(desired_velocity, max_velocity);
    let acceleration = (result_velocity - ground_linear) / time.delta_secs();

    forces.apply_force(acceleration * mass.value());
}

pub fn player_physics() -> impl Bundle {
    (
        LockedAxes::default()
            .lock_rotation_x()
            .lock_rotation_y()
            .lock_rotation_z(),
        Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
    )
}
