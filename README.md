# teph_show_scene

Simple `tephrite_rs` scene viewer for GLTF/GLB files or a TOML scene list.

## Requirements

- Rust toolchain
- A valid Tephrite install on the host platform
- Assets referenced by the input file

## Build

```
cargo build
```

## Run

### GLB/GLTF

```
cargo run -- path/to/scene.glb
```

### TOML scene list

The TOML format allows you to list multiple scenes and an optional environment.
Only the first scene group is visible initially. Use the configured input
controls to switch between groups at runtime.

Example:

```toml
[[scenes]]
content = ["assets/scene_a.glb"]

[[scenes]]
content = ["assets/scene_b.glb", "assets/scene_c.glb"]

[environment]
environment_light_image = "ibl/workshop_4k_small.exr"
environment_light_scale = 5000.0
```

Run:

```
cargo run -- path/to/scenes.toml
```

## Options

- `--environment-light-image <path>`
- `--environment-light-scale <float>`

These CLI options override values from the TOML file when provided.

## Notes

- Group switching uses `JoystickButton::TL` and `JoystickButton::TR`
