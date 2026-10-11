pub use avian3d::collision::prelude::*;
pub use avian3d::prelude::*;
pub use bevy::prelude::*;
use leafwing_input_manager::{Actionlike, InputControlKind};
pub use serde::{Deserialize, Serialize};

use lightyear::{
    core::id::PeerId,
    input::config::InputConfig,
    prelude::{AppComponentExt, input::leafwing::InputPlugin},
};

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
        app.component::<ObjectMarker>().replicate();
        app.component::<Glowing>().replicate();
        app.component::<Light>().replicate();

        // Create new object from markers
        app.add_observer(new_marker);
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

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Reflect)]
#[require(ObjectSize)]
pub enum ObjectType {
    Cuboid,
    Sphere,
    Capsule,
}

impl ObjectType {
    #[cfg(feature = "client")]
    pub fn material(&self) -> StandardMaterial {
        use bevy::color::palettes::css::*;

        match self {
            _ => StandardMaterial {
                base_color: GRAY.into(),
                ..default()
            },
        }
    }

    pub fn mesh(&self, size: Vec3) -> Mesh {
        match self {
            Self::Cuboid => Cuboid::from_size(size).into(),
            Self::Sphere => Sphere::new(size.x).into(),
            Self::Capsule => Capsule3d::new(size.x, size.y).into(),
        }
    }

    pub fn collider(&self, size: Vec3) -> Collider {
        match self {
            Self::Cuboid => Collider::cuboid(size.x, size.y, size.z),
            Self::Sphere => Collider::sphere(size.x),
            Self::Capsule => Collider::capsule(size.x, size.y),
        }
    }
}

/// Object on scene
#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Reflect)]
pub enum ObjectMarker {
    Crate,
    Floor,
    // todo: create much object's
    Player,
}

impl ObjectMarker {
    pub fn bundle(&self) -> impl Bundle {
        match self {
            Self::Crate => {
                let t = ObjectType::Cuboid;
                let l = LockedAxes::new();
                let s = ObjectSize(Vec3::splat(0.5));
                let f = Friction::new(0.6);
                let c = t.collider(s.0);
                let r = RigidBody::Dynamic;

                (t, l, s, f, c, r)
            }
            Self::Floor => {
                let t = ObjectType::Cuboid;
                let l = LockedAxes::new();
                let s = ObjectSize(Vec3::new(10.0, 0.2, 10.0));
                let f = Friction::new(0.6);
                let c = t.collider(s.0);
                let r = RigidBody::Static;

                (t, l, s, f, c, r)
            }
            Self::Player => {
                let t = ObjectType::Capsule;
                let l = LockedAxes::new()
                    .lock_rotation_x()
                    .lock_rotation_y()
                    .lock_rotation_z();
                let s = ObjectSize(Vec2::new(0.5, 1.2).xyx());
                let f = Friction::new(0.6);
                let c = t.collider(s.0);
                let r = RigidBody::Dynamic;

                (t, l, s, f, c, r)
            }
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

pub fn new_marker(
    trigger: On<Add, ObjectMarker>,
    query: Query<&ObjectMarker>,
    mut commands: Commands,
) {
    let marker = query.get(trigger.entity).unwrap();
    commands.entity(trigger.entity).insert(marker.bundle());
}

/// Main inputs data
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
