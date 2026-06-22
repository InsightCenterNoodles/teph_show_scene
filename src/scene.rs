use bevy::prelude::*;
use serde::Deserialize;
use std::path::PathBuf;
use tephrite_rs::prelude::*;

use crate::components::{CurrentGroup, Group, OptionalContent};

#[derive(Debug, Default, Deserialize)]
struct SceneFile {
    scenes: Vec<AScene>,
    environment: Option<EnvironmentOptions>,
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
                ChildOf(group),
            ));
        }

        if let Some(optional_paths) = ascene.optional {
            for content in optional_paths {
                commands.spawn((
                    SceneRoot(server.load_override(GltfAssetLabel::Scene(0).from_asset(content))),
                    Visibility::Hidden,
                    OptionalContent,
                    ChildOf(group),
                ));
            }
        }

        info!("Imported scene {scene_i}");
    }

    Ok(file.environment)
}
