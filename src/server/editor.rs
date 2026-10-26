use bevy::prelude::*;
use doppely::scene::{SceneComponent, SceneObject};
use doppely::*;
use lightyear::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct SceneCounter(u32);

impl SceneCounter {
    pub fn next(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }

    pub fn reset_to(&mut self, value: u32) {
        self.0 = value;
    }
}

pub fn spawn_object(commands: &mut Commands, object: &SceneObject) {
    let dynamic = object.dynamic();

    let mut entity = commands.spawn((
        object.clone(),
        object.kind,
        ObjectSize(object.size),
        Position(object.position),
        Rotation(object.quat()),
        Name::new(object.name.clone()),
        Replicate::to_clients(NetworkTarget::All),
    ));

    if object.kind == ObjectType::Spawnpoint {
        entity.insert(doppely::scene::PlayerSpawnpoint);
        entity.insert(RigidBody::Static);
    } else if dynamic {
        entity.insert(RigidBody::Dynamic);
        entity.insert(PredictionTarget::to_clients(NetworkTarget::All));
    } else {
        entity.insert(RigidBody::Static);
    }

    for component in &object.components {
        match *component {
            SceneComponent::Dynamic => {}
            SceneComponent::Glow => {
                entity.insert(Glowing {
                    color: object.kind.accent(),
                });
            }
            SceneComponent::Light { intensity, radius } => {
                entity.insert(Light {
                    radius,
                    color: object.kind.accent(),
                    intensity,
                });
            }
            SceneComponent::Friction { coefficient } => {
                entity.insert(Friction::new(coefficient));
            }
        }
    }
}

pub fn spawn_scene(
    commands: &mut Commands,
    counter: &mut SceneCounter,
    objects: impl IntoIterator<Item = SceneObject>,
) {
    let mut count = 0;
    for (index, mut object) in objects.into_iter().enumerate() {
        object.id = index as u32;
        spawn_object(commands, &object);
        count += 1;
    }
    counter.reset_to(count);
}

pub fn room(x: f32, z: f32, width: f32, depth: f32, height: f32) -> Vec<SceneObject> {
    let thickness = 0.2;
    let wall = || SceneComponent::Friction { coefficient: 0.8 };

    vec![
        SceneObject::new(
            0,
            ObjectType::Cuboid,
            Vec3::new(width, thickness, depth),
            Vec3::new(x, 0.0, z),
            "Room floor",
        )
        .with(wall()),
        SceneObject::new(
            0,
            ObjectType::Cuboid,
            Vec3::new(width, height, thickness),
            Vec3::new(x, height / 2.0, z - depth / 2.0),
            "Room wall N",
        )
        .with(wall()),
        SceneObject::new(
            0,
            ObjectType::Cuboid,
            Vec3::new(width, height, thickness),
            Vec3::new(x, height / 2.0, z + depth / 2.0),
            "Room wall S",
        )
        .with(wall()),
        SceneObject::new(
            0,
            ObjectType::Cuboid,
            Vec3::new(thickness, height, depth),
            Vec3::new(x - width / 2.0, height / 2.0, z),
            "Room wall W",
        )
        .with(wall()),
        SceneObject::new(
            0,
            ObjectType::Cuboid,
            Vec3::new(thickness, height, depth),
            Vec3::new(x + width / 2.0, height / 2.0, z),
            "Room wall E",
        )
        .with(wall()),
    ]
}
