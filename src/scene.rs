use bevy::prelude::*;
use serde::Deserialize;
use std::path::PathBuf;
use tephrite_rs::prelude::*;

use crate::components::{CurrentGroup, Group};

#[derive(Debug, Default, Deserialize)]
struct SceneFile {
    scenes: Vec<AScene>,
    environment: Option<EnvironmentOptions>,
}

#[derive(Debug, Default, Deserialize)]
struct AScene {
    //title: String,
    content: Vec<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
pub struct EnvironmentOptions {
    pub environment_light_image: PathBuf,

    pub environment_light_scale: Option<f32>,
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
                Replicated,
                PropagateReplication::default(),
                ChildOf(group),
            ));
        }
    }

    Ok(file.environment)
}
