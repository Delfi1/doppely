// Game systems realisation
use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseMotion},
    prelude::*,
    window::*,
};
use doppely::*;
use leafwing_input_manager::{action_state::*, input_map::InputMap, plugin::InputManagerSystem};
use lightyear::input::client::*;

pub const SENSITIVITY: f32 = 0.002;
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.001;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            hide_cursor.run_if(input_just_pressed(KeyCode::Escape)),
        );

        app.add_systems(
            FixedPreUpdate,
            controller_input
                .before(InputSystems::BufferClientInputs)
                .in_set(InputManagerSystem::ManualControl),
        );
    }
}

fn hide_cursor(mut cursor: Single<Mut<CursorOptions>>) {
    let mode = match cursor.grab_mode {
        CursorGrabMode::None => {
            cursor.visible = false;
            CursorGrabMode::Confined
        }
        _ => {
            cursor.visible = true;
            CursorGrabMode::None
        }
    };
    cursor.grab_mode = mode;
}

fn controller_input(
    mut action: Query<&mut ActionState<PlayerAction>, With<InputMap<PlayerAction>>>,
    mut camera: Query<Mut<Transform>, With<Camera3d>>,
    mut mouse: MessageReader<MouseMotion>,
    cursor: Query<Ref<CursorOptions>>,
    mut yaw: Local<f32>,
    mut pitch: Local<f32>,
) {
    let Ok(mut action) = action.single_mut() else {
        return;
    };
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    let Ok(cursor) = cursor.single() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for event in mouse.read() {
        delta += event.delta;
    }

    if !cursor.visible {
        *yaw -= delta.x * SENSITIVITY;
        *pitch -= delta.y * SENSITIVITY;

        *pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    camera.rotation = Quat::from_rotation_x(*pitch);
    action.set_axis_pair(&PlayerAction::Rotation, Vec2::new(*pitch, *yaw));
}
