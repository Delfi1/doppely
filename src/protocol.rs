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
pub enum ObjectMarker {
    Cube,
    Floor,
}

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
#[repr(u8)]
#[cfg_attr(feature = "server", derive(strum::EnumCount))]
pub enum PlayerAction {
    Move = 1,
    Catch = 2,
    Shift = 3,
    MouseMove = 4,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::Move | Self::MouseMove => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
// TODO: придумать больше предметов
pub enum ItemType {
    None = 0,
    Key = 1,
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
        app.component::<Transform>().replicate_with_priority(0);

        app.component::<Light>().replicate();
        app.component::<Glowing>().replicate();

        app.component::<ObjectMarker>().replicate();
    }
}
