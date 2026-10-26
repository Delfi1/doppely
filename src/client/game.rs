use bevy::{input::mouse::MouseMotion, prelude::*, window::*};
use doppely::*;
use leafwing_input_manager::{action_state::*, input_map::InputMap, plugin::InputManagerSystem};
use lightyear::input::client::*;

pub const SENSITIVITY: f32 = 0.002;
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.001;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraLook>().add_systems(
            FixedPreUpdate,
            controller_input
                .before(InputSystems::BufferClientInputs)
                .in_set(InputManagerSystem::ManualControl),
        );
    }
}

#[derive(Resource, Default)]
pub struct CameraLook {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn reset_look(world: &mut World) {
    *world.resource_mut::<CameraLook>() = CameraLook::default();
}

fn controller_input(
    mut action: Query<&mut ActionState<PlayerAction>, With<InputMap<PlayerAction>>>,
    mut mouse: MessageReader<MouseMotion>,
    cursor: Query<Ref<CursorOptions>>,
    mut look: ResMut<CameraLook>,
) {
    let Ok(mut action) = action.single_mut() else {
        return;
    };
    let Ok(cursor) = cursor.single() else {
        return;
    };

    if cursor.grab_mode == CursorGrabMode::None {
        action.reset_all();
        action.set_axis_pair(&PlayerAction::Rotation, Vec2::new(look.pitch, look.yaw));
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse.read() {
        delta += event.delta;
    }

    look.yaw -= delta.x * SENSITIVITY;
    look.pitch = (look.pitch - delta.y * SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);

    action.set_axis_pair(&PlayerAction::Rotation, Vec2::new(look.pitch, look.yaw));
}
