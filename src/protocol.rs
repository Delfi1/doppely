//use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::input::leafwing::prelude::*;
use lightyear::input::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{PLAYER_SPEED, SHIFT_MULTIPLIER};

#[derive(Component, Clone, Debug, PartialEq, Reflect, Deref, DerefMut, Serialize, Deserialize)]
pub struct PlayerId(pub PeerId);

// Input

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
pub enum PlayerAction {
    Up,
    Down,
    Left,
    Right,
    Catch,
    Shift,
    MoveCursor,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::MoveCursor => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

pub fn apply_movement(
    mut query: Query<(&ActionState<PlayerAction>, Mut<Transform>), With<PlayerId>>,
) {
    for (action_state, mut transform) in query.iter_mut() {
        // rotate:

        // move:
        let mut movement = Vec3::ZERO;
        if action_state.pressed(&PlayerAction::Up) {
            movement += *transform.forward();
        }
        if action_state.pressed(&PlayerAction::Down) {
            movement -= *transform.forward();
        }
        if action_state.pressed(&PlayerAction::Left) {
            movement -= *transform.right();
        }
        if action_state.pressed(&PlayerAction::Right) {
            movement += *transform.right();
        }

        if movement != Vec3::ZERO {
            movement = movement.normalize() * PLAYER_SPEED;
            if action_state.pressed(&PlayerAction::Shift) {
                movement *= SHIFT_MULTIPLIER;
            }

            transform.translation += movement;
        }
    }
}

/// Channels
pub struct Channel1;

#[derive(
    States, Resource, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, Reflect,
)]
pub enum GameState {
    #[default]
    Lobby,
    InGame,
    Editor,
}

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<PlayerAction> {
            config: InputConfig {
                lag_compensation: true,
                ..default()
            },
        });

        // components
        app.component::<PlayerId>().replicate();
        app.component::<Transform>().replicate().predict();

        // resources
        app.resource::<GameState>().replicate();
        app.init_state::<GameState>();
        app.init_resource::<GameState>();

        // channels
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ServerToClient);
    }
}
