# Bevy-GUI

A plugin-first editor platform built on Bevy 0.19. The project is structured as an extensible editor foundation for a Godot-class workflow: the application shell wires subsystems together, while viewport, UI, scene, project, runtime and command services stay independently replaceable.

## Architecture

```text
src/
├── app.rs          # dependency wiring and built-in command registration only
├── editor.rs       # editor/plugin API and editor state
├── panel.rs        # panel registration service
├── command.rs      # command definitions, registry and bus
├── docking.rs      # dock layout and panel rendering adapter
├── viewport.rs     # 3D world, camera, picking, gizmos, shortcuts, play mode
├── ui.rs           # egui editor system and authoring actions
├── history.rs      # transform undo/redo history
├── project.rs      # project manifest and persistence
├── scene.rs        # scene document format and serialization
├── runtime.rs      # play-session snapshots
├── selection.rs    # single/multi entity selection state
└── plugins/
    └── mod.rs      # built-in editor plugin installation
```

The important boundary is `app.rs`: it should configure systems, not contain editor behavior. Viewport behavior belongs in `viewport.rs`, UI behavior belongs in `ui.rs`, and persistent domain models belong in their own services.

## Current platform

- 3D editor viewport foundation with Bevy FreeCamera and InfiniteGrid
- Picking and Transform Gizmo integration
- Translate / Rotate / Scale and World / Local switching
- Multi-selection in the scene hierarchy
- Create, Duplicate and Delete entity authoring
- Transform inspector with undo/redo history
- Docking workspace with Viewport, Hierarchy, Inspector, Assets, Console and Plugins
- Plugin registry, panel registry, command registry and command bus
- Project manifest persistence
- Scene JSON serialization and round-trip tests
- Play / Pause / Stop session snapshots
- Asset filesystem browser
- Manual GitHub Actions validation for check, format, clippy and tests

## Design rules

The editor core does not own game-specific components. Extensions communicate through plugins, commands, registries, resources and explicit subsystem APIs. Clippy workarounds are not used as a substitute for architecture; warnings should normally be fixed in the owning subsystem.

## Build

```bash
cargo check --all-targets --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

The GitHub Actions workflow is intentionally `workflow_dispatch`-only, so validation runs only when manually requested.
