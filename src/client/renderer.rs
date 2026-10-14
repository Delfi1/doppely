use bevy::color::palettes::css;
pub use bevy::diagnostic::*;
use bevy::prelude::*;
use bevy::{camera::visibility::*, post_process::bloom::Bloom};
use bevy_lunex::prelude::*;
use doppely::*;
use lightyear::prediction::Predicted;
use lightyear::prelude::Controlled;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, display_debug);
        app.add_observer(handle_objects);
        app.add_observer(add_interpolation);

        if !app.is_plugin_added::<FrameInterpolationPlugin>() {
            app.add_plugins(FrameInterpolationPlugin);
        }

        app.add_observer(attach_menu);
        //app.add_observer(attach_hud);
    }
}

#[derive(Component)]
pub struct DebugText;

pub fn display_debug(
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    player: Option<Single<(Entity, Ref<Transform>), With<Controlled>>>,
    text: Option<Single<Mut<Text>, With<DebugText>>>,
) {
    let Some(mut text) = text else {
        commands.spawn((Text::default(), DebugText));
        return;
    };

    let Some(player) = player else {
        return;
    };

    let (entity, transform) = player.into_inner();

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0)
        .round() as u64;

    let data = format!(
        "Fps: {}\nPlayer entity: {:?}\nCoords: {:.3}",
        fps, entity, transform.translation
    );
    bevy::log::debug!("{}", data);
    *text.as_deref_mut() = data;
}

pub fn handle_objects(
    trigger: On<Add, ObjectType>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    object: Query<
        (
            Ref<ObjectType>,
            Ref<ObjectSize>,
            Option<Ref<Glowing>>,
            Option<Ref<Light>>,
        ),
        Without<Controlled>,
    >,
) {
    let entity = trigger.entity;
    let Ok((object, size, glowing, light)) = object.get(entity) else {
        return;
    };

    let mesh = meshes.add(object.mesh(size.0));
    let mut material = object.material();

    if let Some(glowing) = glowing {
        material.emissive = LinearRgba::from(glowing.color);
    }

    let material = materials.add(material.clone());
    commands
        .entity(entity)
        .insert((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())));

    if let Some(light) = light {
        commands.entity(entity).insert((
            PointLight {
                radius: light.radius,
                intensity: light.intensity,
                ..default()
            },
            NoFrustumCulling,
        ));
    }
}

fn add_interpolation(
    trigger: On<Add, Position>,
    query: Query<&RigidBody, With<Predicted>>,
    clients: Query<(), With<lightyear::connection::client::Client>>,
    mut commands: Commands,
) {
    if clients.is_empty() {
        return;
    }
    let Ok(rb) = query.get(trigger.entity) else {
        return;
    };

    if rb.is_static() {
        return;
    }

    commands.entity(trigger.entity).insert(FrameInterpolate);
}

// UI

fn attach_menu(
    trigger: On<Add, Camera3d>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity).with_children(|camera| {
        camera
            .spawn((
                UiRoot3d,
                UiLayoutRoot::new_3d(),
                Dimension::from((0.5, 0.2)),
                Transform::from_xyz(-0.25, 0.0, -0.8)
                    .with_rotation(Quat::from_rotation_y(40.0_f32.to_radians())),
            ))
            .with_children(|ui| {
                ui.spawn((
                    Name::new("Panel"),
                    UiLayout::solid().size((818.0, 171.0)).pack(),
                    UiMeshPlane3d,
                    OnHoverSetCursor::new(SystemCursorIcon::Pointer),
                ))
                .with_children(|ui| {
                    ui.spawn((
                        Name::new("Text"),
                        UiLayout::window()
                            .x(Rl(20.0))
                            .y(Rl(45.0))
                            .anchor(Anchor::CENTER_LEFT)
                            .pack(),
                        UiTextSize::from(Rh(20.0)),
                        Text3d::new("Testing 3d HUD Text"),
                        TextAnimator::new("Testing 3d HUD Text"),
                        Text3dStyling {
                            size: 128.0,
                            color: css::MISTY_ROSE,
                            align: TextAlign::Center,
                            weight: Weight::BOLD,
                            ..Default::default()
                        },
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..Default::default()
                        })),
                        Mesh3d::default(),
                        OnHoverSetCursor::new(SystemCursorIcon::Pointer),
                    ))
                    .observe(|_: On<Pointer<Out>>| info!("Moving out!"))
                    .observe(|_: On<Pointer<Over>>| info!("Moving in!"))
                    .observe(|_: On<Pointer<Click>>| info!("Click!"));
                });
            });

        camera
            .spawn((
                UiRoot3d,
                UiLayoutRoot::new_3d(),
                Dimension::from((0.5, 0.2)),
                Transform::from_xyz(0.25, 0.0, -0.8)
                    .with_rotation(Quat::from_rotation_y(-40.0_f32.to_radians())),
            ))
            .with_children(|ui| {
                ui.spawn((
                    Name::new("Panel"),
                    UiLayout::solid().size((818.0, 171.0)).pack(),
                    UiMeshPlane3d,
                    OnHoverSetCursor::new(SystemCursorIcon::Pointer),
                ))
                .with_children(|ui| {
                    ui.spawn((
                        Name::new("Text"),
                        UiLayout::window()
                            .x(Rl(20.0))
                            .y(Rl(45.0))
                            .anchor(Anchor::CENTER_LEFT)
                            .pack(),
                        UiTextSize::from(Rh(20.0)),
                        Text3d::new("Doppely game: v0.5"),
                        TextAnimator::new("Doppely game: v0.5"),
                        Text3dStyling {
                            size: 128.0,
                            color: css::DARK_GOLDENROD,
                            align: TextAlign::Center,
                            weight: Weight::BOLD,
                            ..Default::default()
                        },
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..Default::default()
                        })),
                        Mesh3d::default(),
                        OnHoverSetCursor::new(SystemCursorIcon::Pointer),
                    ))
                    .observe(|_: On<Pointer<Out>>| info!("Moving out!"))
                    .observe(|_: On<Pointer<Over>>| info!("Moving in!"))
                    .observe(|_: On<Pointer<Click>>| info!("Click!"));
                });
            });
    });
}

//fn attach_hud(trigger: On<Add, Camera3d>) {}

// Cameras
pub fn controller_camera() -> impl Bundle {
    (Camera3d::default(), Bloom::default(), UiSourceCamera::<1>)
}

pub fn menu_camera() -> impl Bundle {
    (Camera2d::default(), UiSourceCamera::<0>)
}
