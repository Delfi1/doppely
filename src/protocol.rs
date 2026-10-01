pub use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::input::leafwing::prelude::*;
use lightyear::input::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, PartialEq, Reflect, Deref, DerefMut, Serialize, Deserialize)]
pub struct PlayerId(pub PeerId);

// Objects:

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct FloorMarker;

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct CubeMarker;

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
/// Object glowing Marker
pub struct Glowing {
    pub color: Srgba,
}

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
/// Point light Marker
pub struct Light {
    pub radius: f32,
    pub color: Srgba,
    pub intensity: f32,
}

// Input

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
pub enum PlayerAction {
    Forward,
    Backward,
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

/// Channels
pub struct Channel1;

#[derive(
    States, Resource, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default, Reflect,
)]
pub enum GameState {
    Loading,
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
                rebroadcast_inputs: true,
                ..default()
            },
        });

        // components
        app.component::<PlayerId>().replicate();
        app.component::<Name>().replicate();

        app.component::<RigidBody>().replicate();
        app.component::<Collider>().replicate();
        app.component::<LockedAxes>().replicate();
        app.component::<Friction>().replicate();
        app.component::<Transform>()
            .replicate_with_priority(0)
            .predict();

        app.component::<Light>().replicate();
        app.component::<Glowing>().replicate();

        app.component::<CubeMarker>().replicate();
        app.component::<FloorMarker>().replicate();

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
