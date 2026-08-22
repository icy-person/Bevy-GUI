# Bevy-GUI

A plugin-first, native Rust editor shell for the Bevy game engine.

## Architecture

The editor kernel is intentionally small. Features are installed as Bevy plugins and discoverable through registries rather than being hard-coded into one monolithic UI.

- **EditorPlugin** — lifecycle boundary for editor capabilities.
- **PanelRegistry** — extension point for dockable/editor panels.
- **EditorCommandRegistry** — command palette, menus and shortcuts can be layered on top.
- **SelectionState** — shared cross-panel selection model.
- **ProjectState** — serializable project/session state.
- **Built-in plugins** — scene, viewport, inspector, asset browser and console.

## Current stack

- Bevy 0.19
- `bevy_egui` 0.41
- egui 0.36
- Rust 2024 edition
- Native GPU path through Bevy/wgpu

## Direction

The project is intended to grow into a modular Bevy editor rather than a demo UI. The next layers are scene serialization, real 3D viewport rendering/picking, transform gizmos, asset indexing and hot reload, reflection-driven component inspectors, undo/redo transactions, project settings, profiler tooling, plugin discovery, and Android build/deploy tooling.

## Run

```bash
cargo run --release
```

The editor is expected to remain usable as a host application while individual editor plugins can later be disabled or replaced by downstream projects.
