use std::path::{Path, PathBuf};

use bevy::prelude::{Component, EulerRot, Quat, Reflect, Vec3};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ObjectType;

pub const SCENES_DIR: &str = "scenes";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Reflect, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub enum SceneComponent {
    Dynamic,
    Glow,
    Light { intensity: f32, radius: f32 },
    Friction { coefficient: f32 },
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerSpawnpoint;

#[derive(Component, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SceneObject {
    pub id: u32,
    pub kind: ObjectType,
    pub size: Vec3,
    pub position: Vec3,
    #[serde(default)]
    pub rotation: Vec3,
    pub name: String,
    #[serde(default)]
    pub components: Vec<SceneComponent>,
}

impl SceneObject {
    pub fn new(id: u32, kind: ObjectType, size: Vec3, position: Vec3, name: &str) -> Self {
        Self {
            id,
            kind,
            size,
            position,
            rotation: Vec3::ZERO,
            name: name.to_string(),
            components: Vec::new(),
        }
    }

    pub fn with(mut self, component: SceneComponent) -> Self {
        self.components.push(component);
        self
    }

    pub fn dynamic(&self) -> bool {
        self.components
            .iter()
            .any(|component| matches!(component, SceneComponent::Dynamic))
    }

    pub fn quat(&self) -> Quat {
        Quat::from_euler(
            EulerRot::XYZ,
            self.rotation.x.to_radians(),
            self.rotation.y.to_radians(),
            self.rotation.z.to_radians(),
        )
    }
}

pub fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(SCENES_DIR)
}

pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn path_for(name: &str) -> Option<PathBuf> {
    is_valid_name(name).then(|| dir().join(format!("{name}.json")))
}

pub fn scenes() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir()) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                return None;
            }
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
        })
        .collect();

    names.sort();
    names
}

pub fn load(name: &str) -> Result<Vec<SceneObject>, String> {
    let Some(path) = path_for(name) else {
        return Err(format!(
            "Недопустимое имя сцены «{name}»: только латиница, цифры, «_» и «-»"
        ));
    };

    let data = std::fs::read_to_string(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            format!("Сцена «{name}» не найдена ({})", path.display())
        } else {
            format!("Не удалось прочитать {}: {error}", path.display())
        }
    })?;

    serde_json::from_str(&data)
        .map_err(|error| format!("Сцена {} повреждена ({error})", path.display()))
}

pub fn save(name: &str, objects: &[SceneObject]) -> Result<PathBuf, String> {
    let Some(path) = path_for(name) else {
        return Err(format!(
            "Недопустимое имя сцены «{name}»: только латиница, цифры, «_» и «-»"
        ));
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Не удалось создать {}: {error}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(objects)
        .map_err(|error| format!("Не удалось сериализовать сцену: {error}"))?;
    std::fs::write(&path, format!("{json}\n"))
        .map_err(|error| format!("Не удалось записать {}: {error}", path.display()))?;

    Ok(path)
}

pub fn default_components() -> Vec<SceneComponent> {
    vec![SceneComponent::Friction { coefficient: 0.8 }]
}

pub fn default_size(kind: ObjectType) -> Vec3 {
    match kind {
        ObjectType::Cuboid => Vec3::ONE,
        ObjectType::Sphere => Vec3::splat(0.5),
        ObjectType::Capsule => Vec3::new(0.25, 1.0, 0.25),
        ObjectType::Cylinder => Vec3::new(0.5, 1.0, 0.5),
        ObjectType::Cone => Vec3::new(0.5, 1.0, 0.5),
        ObjectType::Spawnpoint => Vec3::splat(0.5),
    }
}

pub fn ground_y(kind: ObjectType, size: Vec3) -> f32 {
    match kind {
        ObjectType::Sphere => size.x,
        ObjectType::Capsule => size.y * 0.5 + size.x,
        ObjectType::Spawnpoint => 1.1,
        _ => size.y * 0.5,
    }
}

pub fn empty_scene() -> Vec<SceneObject> {
    vec![SceneObject::new(
        0,
        ObjectType::Cuboid,
        Vec3::ONE,
        Vec3::new(0.0, 0.5, 0.0),
        "Cube",
    )]
}
