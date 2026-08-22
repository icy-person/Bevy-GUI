# Bevy-GUI

Bevy-GUI is a plugin-first game-editor platform built on Bevy 0.19. The codebase is organized as independent editor subsystems rather than a single UI file, with the long-term target of a Godot-class workflow.

## Implemented systems

- 3D viewport foundation with Bevy FreeCamera and InfiniteGrid
- Picking and Transform Gizmo integration
- Translate / Rotate / Scale and World / Local switching
- Multi-selection and tree-style scene hierarchy
- Create / Duplicate / Delete entity authoring
- Parent / Unparent authoring
- Transform inspector with undo/redo history
- Versioned scene JSON with stable IDs and parent relationships
- Project manifest loading before editor startup
- Main-scene loading on editor startup
- Play / Pause / Stop runtime snapshots
- Command Bus and command executor
- Asset database with classification, file sizes and modification metadata
- Docking workspace with Viewport, Hierarchy, Inspector, Assets, Console and Plugins
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
├── ui/
│   ├── mod.rs
│   ├── actions.rs
│   ├── assets.rs
│   └── persistence.rs
│
├── viewport/
│   ├── mod.rs
│   ├── components.rs
│   ├── scene.rs
│   ├── input.rs
│   ├── gizmo.rs
│   └── runtime.rs
│
└── plugins/
    ├── mod.rs
    ├── scene.rs
    ├── viewport.rs
    ├── inspector.rs
    ├── assets.rs
    └── console.rs
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
