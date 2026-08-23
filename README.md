# Bevy-GUI

Bevy-GUI is a plugin-first game-editor platform built on Bevy 0.19. The editor is organized as independent subsystems so project management, scene authoring, UI, runtime control and build/export can evolve without a monolithic editor file.

## Real editor capabilities

- Material 3-inspired Welcome / Project Manager
- New Project creates a persistent runnable Bevy project on disk
- Generated project contains `project.godot-rs.json`, `Cargo.toml`, `src/main.rs`, `scenes/`, `assets/` and `.bevy-gui/`
- Open Project loads the real manifest and project root
- 3D viewport with FreeCamera, InfiniteGrid, picking and Transform Gizmo
- 3D Translate / Rotate / Scale and World / Local switching
- 2D viewport with Camera2d, orthographic pan/zoom and grid
- 2D / 3D switching with `1` / `2`
- Multi-selection and hierarchy tree
- Create / Duplicate / Delete entity authoring
- Parent / Unparent authoring
- Live Inspector editing for Transform, Name and Visibility
- Versioned scene JSON with stable IDs, parent relationships and visibility state
- Main Scene persistence across restart
- Play / Pause / Stop runtime mode
- Command Bus and command executor
- Searchable Asset Browser with type classification, size, generation and selection
- Persistent settings with Material Light/Dark styling, UI scale, keymap, viewport, graphics and project controls
- Docking workspace for Viewport, Hierarchy, Inspector, Assets, Console, Profiler, Plugins and Settings
- Live profiler with FPS and frame-time statistics
- Plugin and panel registries with live service diagnostics
- Export pipeline capable of invoking `cargo build --release` and packaging the runtime executable, scene and assets
- Manual Linux x86_64 release/debug build workflow

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
Ctrl+Shift+B       Build/export project
Ctrl+O             Open scene command
Ctrl+Shift+S       Save scene command
```

## Project persistence

New projects contain a real Bevy 0.19 Cargo game template. The editor writes scene data to `scenes/main.scene.json` and settings to `.bevy-gui/editor-settings.json`.

## Validation

```bash
cargo check --all-targets --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

GitHub Actions is intentionally manual-only (`workflow_dispatch`). Use **Actions → build → Run workflow → release** when you want a new editor binary.
