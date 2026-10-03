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
    Player,
}

impl ObjectMarker {
    pub fn mesh(&self) -> Mesh {
        match self {
            ObjectMarker::Cube => Cuboid::from_length(1.0).into(),
            ObjectMarker::Floor => Cuboid::from_size([10.0, 0.2, 10.0].into()).into(),
            ObjectMarker::Player => Capsule3d::new(0.5, 1.2).into(),
        }
    }

    #[cfg(feature = "client")]
    pub fn material(&self) -> StandardMaterial {
        use bevy::color::palettes::css::*;

        match self {
            ObjectMarker::Cube | ObjectMarker::Floor | ObjectMarker::Player => StandardMaterial {
                base_color: GRAY.into(),
                ..default()
            },
        }
    }

    pub fn collider(&self) -> Collider {
        match self {
            ObjectMarker::Cube => Collider::cuboid(1.0, 1.0, 1.0),
            ObjectMarker::Floor => Collider::cuboid(10.0, 0.2, 10.0),
            ObjectMarker::Player => Collider::capsule(0.5, 1.2),
        }
    }
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

// Мировые объекты, и предметы (инвентарь)

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
// TODO: придумать больше предметов
pub enum Item {
    None = 0,
    Key = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
// TODO: придумать больше объектов
pub enum Object {
    None = 0,
    Crate = 1,
}

pub type Inventory = [Item; 3];

// Состояния сервера

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnterLobby {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnterEditor {}

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
        app.component::<Position>().replicate().predict();
        app.component::<Rotation>().replicate().predict();

        app.component::<Light>().replicate();
        app.component::<Glowing>().replicate();

        app.component::<RigidBody>().replicate();
        app.component::<Collider>().replicate();
        app.component::<LockedAxes>().replicate();
        app.component::<Friction>().replicate();

        app.component::<ObjectMarker>().replicate();
    }
}
