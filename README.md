# Bevy-GUI

Bevy-GUI is a plugin-first game-editor platform built on Bevy 0.19. The codebase is organized as independent editor subsystems rather than a single UI file, with the long-term target of a Godot-class workflow.

## Implemented editor systems

- Material 3-inspired editor shell with Welcome/Project entry, app bar and navigation rail
- 3D viewport with FreeCamera, InfiniteGrid, mesh picking and Transform Gizmo
- 3D Translate / Rotate / Scale and World / Local switching
- Configurable 3D grid visibility and transform snapping
- 2D viewport with Camera2d, orthographic zoom, middle-mouse pan and configurable grid
- One-click and keyboard switching between 2D and 3D (`1` / `2`)
- Multi-selection and tree-style scene hierarchy
- Create / Duplicate / Delete entity authoring
- Parent / Unparent authoring
- Transform inspector with undo/redo history
- Versioned scene JSON with stable IDs and parent relationships
- Project manifest loading before editor startup
- Main-scene loading on editor startup
- Play / Pause / Stop runtime snapshots
- Command Bus and command executor with scene authoring commands
- Asset database with classification, file sizes and modification metadata
- Docking workspace with Viewport, Hierarchy, Inspector, Assets, Console, Profiler, Plugins and Settings
- Live Profiler panel with FPS, frame time, min/max frame timing and sample count
- Persistent settings with versioned JSON storage
- Settings categories for Appearance, Editor, Viewport/Grid, Input, Graphics and Project
- Runtime-configurable camera speed, zoom/pan behavior, grid, snapping and Material appearance
- Project export pipeline for manifest, scene and assets
- Plugin and panel registries
- Focused subsystem file layout
- Manual Linux x86_64 release/debug build artifacts

## Source layout

```text
src/
├── app.rs
├── lib.rs
├── command.rs
├── command_executor.rs
├── editor.rs
├── export.rs
├── history.rs
├── panel.rs
├── profiler.rs
├── project.rs
├── runtime.rs
├── scene.rs
├── scene_model.rs
├── selection.rs
│
├── assets/
│   ├── mod.rs
│   └── database.rs
│
├── docking/
│   ├── mod.rs
│   ├── state.rs
│   └── viewer.rs
│
├── settings/
│   └── mod.rs
│
├── ui/
│   ├── mod.rs
│   ├── actions.rs
│   ├── assets.rs
│   ├── persistence.rs
│   ├── settings.rs
│   ├── theme.rs
│   ├── welcome.rs
│   └── workspace.rs
│
├── viewport/
│   ├── mod.rs
│   ├── components.rs
│   ├── scene.rs
│   ├── input.rs
│   ├── gizmo.rs
│   └── runtime.rs
│
├── viewport2d/
│   └── mod.rs
│
└── plugins/
    ├── mod.rs
    ├── scene.rs
    ├── viewport.rs
    ├── inspector.rs
    ├── assets.rs
    └── console.rs
```

## Keyboard authoring

```text
1                  Switch to 2D
2                  Switch to 3D
W / E / R          Translate / Rotate / Scale gizmo
X                  Toggle World / Local
Ctrl+Z / Ctrl+Y    Undo / Redo
Ctrl+A             Create entity
Ctrl+D             Duplicate selected entity
Delete             Delete selected entity
F6 / F7 / F8       Play / Pause / Stop
F5                 Refresh assets
Ctrl+S             Save project
Ctrl+Shift+B       Export project
Ctrl+O             Open scene command
Ctrl+Shift+S       Save scene command
```

## Settings persistence

Editor settings are stored at:

```text
.bevy-gui/editor-settings.json
```

The document is versioned and contains:

```text
Appearance
Editor Behavior
Viewport & Grid
Input
Graphics
Project
```

## Design rules

The editor core does not own game-specific components. Cross-subsystem communication goes through resources, commands, registries, selection state and scene/project documents. Large systems are split into focused files so new features do not require rewriting `app.rs` or a monolithic UI module.

## Build checks

```bash
cargo check --all-targets --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Build an editor binary

The `build` GitHub Actions workflow is manual-only. Run **Actions → build → Run workflow**, choose `release` or `debug`, and download the resulting Linux x86_64 artifact from that workflow.

For local Linux builds, see [`docs/build-and-test.md`](docs/build-and-test.md).

GitHub Actions is intentionally manual-only (`workflow_dispatch`) so validation and binary builds run only when requested.
