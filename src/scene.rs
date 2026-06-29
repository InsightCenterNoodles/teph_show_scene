use bevy::{image::ImageLoaderSettings, prelude::*};
use serde::Deserialize;
use std::path::PathBuf;
use tephrite_rs::prelude::*;

use crate::components::{ContentIndex, CurrentGroup, Group};

#[derive(Debug, Default, Deserialize)]
struct InfoGraphic {
    path: PathBuf,
    location: Vec3,
    scale: Option<f32>,
    normal: Option<Vec3>,
}

#[derive(Debug, Default, Deserialize)]
struct SceneFile {
    scenes: Vec<AScene>,
    environment: Option<EnvironmentOptions>,
    info_graphic: Option<InfoGraphic>,
}

#[derive(Debug, Default, Deserialize)]
struct AScene {
    title: Option<String>,

    content: Vec<PathBuf>,

    // Optional content that be shown with a button press
    optional: Option<Vec<PathBuf>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct EnvironmentOptions {
    pub environment_light_image: Option<PathBuf>,

    pub environment_light_scale: Option<f32>,

    pub directional_light_scale: Option<f32>,
}

pub fn import_gltf(p: PathBuf, root: Entity, commands: &mut Commands, server: &mut AssetServer) {
    commands.spawn((
        SceneRoot(server.load_override(GltfAssetLabel::Scene(0).from_asset(p))),
        Replicated,
        PropagateReplication::default(),
        ChildOf(root),
    ));
}

pub fn import_scene(
    p: PathBuf,
    root: Entity,
    commands: &mut Commands,
    server: &mut AssetServer,
    mat_assets: &mut Assets<StandardMaterial>,
    mesh_assets: &mut Assets<Mesh>,
    image_assets: &mut Assets<Image>,
) -> Result<Option<EnvironmentOptions>> {
    let file = std::fs::read(p)?;

    let file: SceneFile = toml::from_slice(&file)?;

    for (scene_i, ascene) in file.scenes.into_iter().enumerate() {
        let group = commands
            .spawn((
                Group {
                    order: scene_i as u32,
                },
                Visibility::Hidden,
                Replicated,
                PropagateReplication::default(),
                ChildOf(root),
            ))
            .id();

        if scene_i == 0 {
            commands
                .entity(group)
                .insert((Visibility::Visible, CurrentGroup));
        }

        for content in ascene.content {
            commands.spawn((
                SceneRoot(server.load_override(GltfAssetLabel::Scene(0).from_asset(content))),
                Visibility::Inherited,
                ContentIndex(0),
                ChildOf(group),
            ));
        }

        if let Some(optional_paths) = ascene.optional {
            for (content_i, content) in optional_paths.into_iter().enumerate() {
                commands.spawn((
                    SceneRoot(server.load_override(GltfAssetLabel::Scene(0).from_asset(content))),
                    Visibility::Hidden,
                    ContentIndex(content_i as u32),
                    ChildOf(group),
                ));
            }
        }

        info!("Imported scene {scene_i}");
    }

    if let Some(info_graphic) = file.info_graphic {
        let image = server.load_with_settings_override(
            info_graphic.path.clone(),
            |settings: &mut ImageLoaderSettings| {
                settings
                    .sampler
                    .get_or_init_descriptor()
                    .set_filter(bevy::image::ImageFilterMode::Linear);
            },
        );

        let mat = mat_assets.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            ..Default::default()
        });

        let mesh = mesh_assets.add(Plane3d {
            normal: Dir3::Z,
            half_size: vec2(0.5, 0.5),
        });

        let rot = Quat::look_at_lh(Vec3::ZERO, info_graphic.normal.unwrap_or(Vec3::Z), Vec3::Y);

        // we may not have the asset size available yet. Defer sizing.

        let mut tf = Transform::from_translation(info_graphic.location).with_rotation(rot);

        if let Some(resolved) = image_assets.get(&image) {
            finalize_image(resolved, &mut tf, info_graphic.scale.unwrap_or(1.0));
        }

        commands.spawn((
            tf,
            MeshMaterial3d(mat),
            Mesh3d(mesh),
            IsInfoGraphic(image, info_graphic.scale.unwrap_or(1.0)),
            Replicated, // is not to be part of the controlled navigation
        ));
    }

    Ok(file.environment)
}

fn finalize_image(image: &Image, transform: &mut Transform, scale: f32) {
    let ratio = image.size_f32() / image.size_f32().max_element();
    let ratio = ratio * scale;

    transform.scale = vec3(ratio.x, ratio.y, 1.0);

    info!("Finalizing infographic {transform:?}");
}

#[derive(Debug, Default, Component)]
pub struct IsInfoGraphic(Handle<Image>, f32);

pub fn check_infographic_updates(
    mut asset_events: MessageReader<AssetEvent<Image>>,
    mut q_infographic_check: Query<(&mut Transform, &IsInfoGraphic)>,
    image_assets: Res<Assets<Image>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                // should be a short loop
                for mut infographic in &mut q_infographic_check {
                    if infographic.1.0.id() == *id {
                        if let Some(image) = image_assets.get(*id) {
                            finalize_image(image, &mut infographic.0, infographic.1.1);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
