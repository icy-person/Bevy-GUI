# Bevy-GUI

A plugin-first editor platform built on Bevy 0.19. The project targets a Godot-class workflow rather than a fixed demo UI: projects, scenes, assets, authoring tools, runtime play sessions, commands and panels are separate extensible subsystems.

## Current platform

- 3D editor viewport foundation with Bevy FreeCamera and InfiniteGrid
- Bevy picking and Transform Gizmo integration
- Translate / Rotate / Scale and World / Local switching
- Multi-selection in the scene hierarchy
- Create, Duplicate and Delete entity authoring
- Transform inspector with undo/redo history
- Docking workspace with Viewport, Hierarchy, Inspector, Assets, Console and Plugins
- Plugin registry and independent panel registry
- Command registry and editor command bus
- Project manifest persistence (`project.godot-rs.json`)
- Scene JSON serialization and round-trip tests
- Play / Pause / Stop session snapshots
- Asset filesystem browser
- CI checks for compile, formatting, clippy and tests

## Architecture

```text
bevy-gui
├── editor core
│   ├── plugin API
│   ├── panel registry
│   ├── command registry / bus
│   ├── selection
│   └── transform history
├── project system
│   ├── manifest
│   ├── main scene
│   └── project settings
├── scene system
│   ├── document format
│   ├── save/load
│   └── SceneNode authoring marker
├── runtime
│   └── isolated PlaySession state
└── UI plugins
    ├── viewport
    ├── scene hierarchy
    ├── inspector
    ├── asset browser
    └── console
```

## Design rule

The editor core does not own game-specific components. Features are exposed through plugins, commands, registries and resources so external editor extensions can add panels, tools and importers without rewriting the core.

## Build

```bash
cargo check --all-targets --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

The Linux CI runner installs the native graphics/audio development libraries required by Bevy and its windowing stack.
