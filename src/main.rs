mod components;
mod scene;

use std::path::PathBuf;

use bevy::prelude::*;
use clap::Parser;
use tephrite_rs::prelude::*;

use crate::components::{CurrentGroup, Group};

#[derive(Debug, clap::Parser)]
#[command(version, about)]
struct Arguments {
    /// Input file
    input: PathBuf,

    #[command(flatten)]
    options: EnvironmentOptions,
}

#[derive(Debug, Default, Clone, clap::Args)]
pub struct EnvironmentOptions {
    #[arg(long)]
    pub environment_light_image: Option<PathBuf>,

    #[arg(long)]
    pub environment_light_scale: Option<f32>,
}

struct LoadScenePlugin;

impl Plugin for LoadScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);

        app.add_observer(on_global_activate);

        app.add_plugins(NavigationPlugin::new(NavigatorMode::ObjectCentric));
    }
}

fn setup(mut commands: Commands, mut server: ResMut<AssetServer>) -> Result<()> {
    // light
    commands.spawn((
        DirectionalLight {
            color: Color::srgb_u8(255, 224, 141),
            shadows_enabled: true,
            illuminance: 130000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 4.0, 3.0).looking_at((0.0, 0.0, 0.0).into(), Dir3::Y),
        Replicated,
    ));

    let mut env_opts = None;

    let root = commands
        .spawn((Transform::default(), Replicated, NavigatorMarker))
        .id();

    let args = Arguments::parse();

    if let Some(ext) = args.input.extension() {
        match ext.to_str() {
            Some("toml") => {
                env_opts = scene::import_scene(args.input, root, &mut commands, &mut server)?;
            }
            Some("glb") | Some("gltf") => {
                scene::import_gltf(args.input, root, &mut commands, &mut server);
            }
            Some(x) => {
                error!("Unsupported file type {x}");
            }
            _ => {
                error!("No file specified!")
            }
        }
    }

    let env_intensity = env_opts
        .as_ref()
        .and_then(|x| x.environment_light_scale)
        .or(args.options.environment_light_scale)
        .unwrap_or(5000.0);

    let env_path = env_opts
        .map(|x| x.environment_light_image)
        .or(args.options.environment_light_image)
        .unwrap_or(PathBuf::from("ibl/workshop_4k_small.exr"));

    {
        let env_map = server.load(env_path);

        commands.insert_resource(EnvironmentLighting {
            intensity: env_intensity,
            equirect: env_map,
        });
    }

    Ok(())
}

fn on_global_activate(
    trigger: On<GlobalActivate>,
    current: Query<(Entity, &CurrentGroup)>,
    known: Query<(Entity, &Group)>,
    mut commands: Commands,
) {
    let mut v: Vec<_> = known.iter().collect();

    v.sort_unstable_by_key(|f| f.1.order);

    let Ok(current) = current.single() else {
        info!("No current group");
        return;
    };

    let Some(current_place) = v.iter().position(|x| x.0 == current.0) else {
        info!("Cannot find current group in group list");
        return;
    };

    let current_len = v.len();

    let next_place = match trigger.button {
        JoystickButton::TL => (current_place + current_len - 1) % current_len,
        JoystickButton::TR => (current_place + 1) % current_len,
        _ => {
            return;
        }
    };

    let Some(next_ent) = v.get(next_place) else {
        info!("Cannot find next group in group list");
        return;
    };

    commands.entity(current.0).insert(Visibility::Hidden);
    commands.entity(next_ent.0).insert(Visibility::Visible);

    commands.entity(current.0).remove::<CurrentGroup>();
    commands.entity(next_ent.0).insert(CurrentGroup);
}

fn main() {
    tephrite_rs::run(LoadScenePlugin);
}
