// Game systems realisation
use bevy::{input::mouse::MouseMotion, prelude::*, window::*};
use doppely::*;
use leafwing_input_manager::{action_state::*, input_map::InputMap, plugin::InputManagerSystem};
use lightyear::input::client::*;

pub const SENSITIVITY: f32 = 0.002;
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.001;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            controller_input
                .before(InputSystems::BufferClientInputs)
                .in_set(InputManagerSystem::ManualControl),
        );
    }
}

/// Сбрасывает накопленный поворот камеры. Вызывается при входе в игру и при
/// её закрытии: иначе после Esc-меню камера «прыгает» на старый yaw/pitch.
pub fn reset_look(world: &mut World) {
    let mut camera = world.query_filtered::<&mut Transform, With<Camera3d>>();
    for mut transform in camera.iter_mut(world) {
        transform.rotation = Quat::IDENTITY;
    }
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
    let Ok(cursor) = cursor.single() else {
        return;
    };

    // Ввод копится только пока игра захватила курсор: в Esc-меню и вне игры
    // камера не должна поворачиваться от движений мыши.
    if cursor.grab_mode == CursorGrabMode::None {
        *yaw = 0.0;
        *pitch = 0.0;
        action.reset_all();
        return;
    }

    let Ok(mut camera) = camera.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for event in mouse.read() {
        delta += event.delta;
    }

    *yaw -= delta.x * SENSITIVITY;
    *pitch = (*pitch - delta.y * SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);

    camera.rotation = Quat::from_rotation_x(*pitch);
    action.set_axis_pair(&PlayerAction::Rotation, Vec2::new(*pitch, *yaw));
}
