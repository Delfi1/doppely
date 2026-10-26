pub use avian3d::collision::prelude::*;
pub use avian3d::prelude::*;
pub use bevy::prelude::*;
use leafwing_input_manager::{Actionlike, InputControlKind};
pub use serde::{Deserialize, Serialize};
use ts_rs::TS;

use lightyear::{
    core::id::PeerId,
    input::config::InputConfig,
    prelude::{
        AppChannelExt, AppComponentExt, AppMessageExt, ChannelMode, ChannelSettings,
        NetworkDirection, ReliableSettings, input::leafwing::InputPlugin,
    },
};

use crate::scene::SceneObject;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<PlayerAction> {
            config: InputConfig {
                rebroadcast_inputs: true,
                ..default()
            },
        });

        app.component::<Name>().replicate();
        app.component::<PlayerId>().replicate();
        app.component::<ObjectType>().replicate();
        app.component::<ObjectSize>().replicate();
        app.component::<Glowing>().replicate();
        app.component::<Light>().replicate();

        app.component::<RigidBody>().replicate();
        app.component::<Friction>().replicate();
        app.component::<LockedAxes>().replicate();

        app.component::<LobbyInfo>().replicate();
        app.component::<ServerMode>().replicate();
        app.component::<EditorInfo>().replicate();
        app.component::<SceneObject>().replicate();

        app.add_channel::<LobbyChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);
        app.add_channel::<EditorChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<StartGame>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<EditorCommand>()
            .add_direction(NetworkDirection::ClientToServer);

        app.add_systems(FixedPostUpdate, setup_collision);
    }
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone, Reflect)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone, Reflect)]
pub struct ObjectSize(pub Vec3);

impl Default for ObjectSize {
    fn default() -> Self {
        Self(Vec3::ONE)
    }
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Reflect, TS)]
#[require(ObjectSize)]
#[ts(export)]
pub enum ObjectType {
    Cuboid,
    Sphere,
    Capsule,
    Cylinder,
    Cone,
    Spawnpoint,
}

impl ObjectType {
    pub fn base_color(&self) -> Srgba {
        use bevy::color::palettes::css::*;

        match self {
            Self::Cuboid => SLATE_GRAY,
            Self::Sphere => GOLDENROD,
            Self::Capsule => MEDIUM_SEA_GREEN,
            Self::Cylinder => PERU,
            Self::Cone => INDIAN_RED,
            Self::Spawnpoint => AQUA,
        }
    }

    pub fn accent(&self) -> Srgba {
        use bevy::color::palettes::css::*;

        match self {
            Self::Cuboid => LIGHT_STEEL_BLUE,
            Self::Sphere => KHAKI,
            Self::Capsule => PALE_GREEN,
            Self::Cylinder => WHEAT,
            Self::Cone => LIGHT_CORAL,
            Self::Spawnpoint => AQUAMARINE,
        }
    }

    #[cfg(feature = "client")]
    pub fn material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: self.base_color().into(),
            perceptual_roughness: 0.8,
            ..default()
        }
    }

    pub fn mesh(&self, size: Vec3) -> Option<Mesh> {
        Some(match self {
            Self::Cuboid => Cuboid::from_size(size).into(),
            Self::Sphere => Sphere::new(size.x).into(),
            Self::Capsule => Capsule3d::new(size.x, size.y).into(),
            Self::Cylinder => Cylinder::new(size.x, size.y).into(),
            Self::Cone => Cone::new(size.x, size.y).into(),
            Self::Spawnpoint => return None,
        })
    }

    pub fn collider(&self, size: Vec3) -> Option<Collider> {
        Some(match self {
            Self::Cuboid => Collider::cuboid(size.x, size.y, size.z),
            Self::Sphere => Collider::sphere(size.x),
            Self::Capsule => Collider::capsule(size.x, size.y),
            Self::Cylinder => Collider::cylinder(size.x, size.y),
            Self::Cone => Collider::cone(size.x, size.y),
            Self::Spawnpoint => return None,
        })
    }
}

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct Glowing {
    pub color: Srgba,
}

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct Light {
    pub radius: f32,
    pub color: Srgba,
    pub intensity: f32,
}

pub fn setup_collision(
    query: Query<(Entity, &ObjectType, &ObjectSize), Changed<ObjectSize>>,
    mut commands: Commands,
) {
    for (entity, object, size) in query.iter() {
        if let Some(collider) = object.collider(size.0) {
            commands.entity(entity).insert(collider);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
#[repr(u8)]
pub enum PlayerAction {
    Move = 1,
    Rotation = 2,
    Catch = 3,
    Shift = 4,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::Move | Self::Rotation => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct LobbyInfo {
    pub players: u8,
    pub max_players: u8,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ServerMode {
    Lobby,
    Game,
    Edit,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct StartGame {}

pub struct LobbyChannel;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Reflect)]
pub enum EditorAction {
    Save {
        name: String,
    },
    Load {
        name: String,
    },
    Refresh,
    Spawn {
        kind: ObjectType,
        position: Vec3,
    },
    Move {
        id: u32,
        position: Vec3,
    },
    Remove {
        id: u32,
    },
    RemoveMany {
        ids: Vec<u32>,
    },
    SetComponents {
        id: u32,
        components: Vec<crate::scene::SceneComponent>,
    },
    SetTransform {
        id: u32,
        position: Vec3,
        rotation: Vec3,
        size: Vec3,
    },
    Undo,
    Redo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Reflect)]
pub struct EditorCommand {
    pub action: EditorAction,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    pub scene: String,
    pub scenes: Vec<String>,
    pub message: String,
    pub ok: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

pub struct EditorChannel;
