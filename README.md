# Bevy-GUI

Bevy-GUI is a plugin-first **game-engine + editor platform** built on Bevy 0.19. It combines a persistent scene authoring environment with a reusable runtime foundation, asset pipeline, diagnostics services, docking workspace and Bevy-native UI integration.

The project deliberately follows ideas proven by Bevy-focused tools such as Jackdaw and BerryCode while keeping the engine/editor code owned by this repository. Jackdaw contributes the direction of native Bevy editor widgets, modular panels, reflection-friendly inspector editing and a document/ECS synchronization model. BerryCode contributes the IDE-oriented direction: Scene Editor, ECS Inspector, System Graph, Event Monitor, Asset workflow, animation/shader/visual-scripting tooling and Bevy-native development workflow. BerryCode currently targets Bevy 0.18, so it is used as an architectural reference rather than a direct dependency in the Bevy 0.19 engine.

## Engine architecture

```text
Bevy 0.19
   │
   ├── Engine runtime services
   │   ├── EngineSettings
   │   ├── EnginePaths
   │   ├── EngineRuntimePlugin
   │   └── load_runtime_scene()
   │
   ├── Authoring/editor services
   │   ├── SceneDocument / SceneEntity
   │   ├── Prefab system
   │   ├── Selection + history
   │   ├── Asset database/import pipeline
   │   └── Viewport + gizmos
   │
   ├── Tooling services
   │   ├── EngineFeatureRegistry
   │   ├── EngineEventMonitor
   │   ├── EngineGraphRegistry
   │   └── EngineDiagnostics
   │
   └── UI
       ├── Bevy Feathers / Jackdaw native widgets
       ├── egui compatibility shell
       └── Docking workspace
```

The engine layer intentionally does not require the editor UI. `EngineRuntimePlugin` owns runtime paths/settings, while `BevyGuiPlugin` adds editor services on top. This makes the direction suitable for generated standalone games as well as the desktop editor.

## Current editor capabilities

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
- Jackdaw Feathers integration for Bevy-native inspector text fields and event-driven editing
- Engine feature registry for Scene Editor, ECS Inspector, System Graph, Event Monitor, Query Visualizer, State Editor, Animation, Visual Scripting, Shader Graph, Asset Browser and Profiler services
- Bounded runtime event monitor and engine diagnostics resources suitable for live tooling
- Engine graph metadata registry for system dependency visualization
- Standalone runtime plugin and explicit authored-scene loading API

## BerryCode-inspired engine tooling

BerryCode's current architecture is particularly useful as a product-level reference because it treats Bevy as the game engine itself rather than merely a library. Its documented feature set includes a Unity-class Scene Editor, ECS Inspector, System Graph, Event Monitor, Query Visualizer, State Editor, templates, plugin discovery, Animation System, Visual Scripting and Shader Graph. citehttps://github.com/KyosukeIshizu1008/berryscode

Bevy-GUI uses those ideas as engine/editor service boundaries. The implementation remains incremental: the current repository already has the scene, asset, selection, command, viewport, runtime and UI foundations; the feature registry gives the next editor panels stable service identities instead of forcing them into one monolithic UI.

## Jackdaw integration

The current Jackdaw integration follows the newer Bevy-native GUI direction. The upstream editor exposes dedicated Feathers widgets such as buttons, dialogs, inspector fields, numeric inputs, tree views, panels and text editing; Bevy-GUI consumes the `jackdaw_feathers` widget layer through `src/jackdaw_ui.rs` while keeping the existing egui shell and docking system. citehttps://github.com/jbuehler23/jackdaw

The integration is intentionally incremental so panels can migrate one at a time rather than forcing an all-or-nothing UI rewrite.

## Source layout

```text
src/
├── app.rs
├── lib.rs
├── engine.rs
├── engine_features.rs
├── jackdaw_ui.rs
├── command.rs
├── command_executor.rs
├── component_registry.rs
├── editor.rs
├── export.rs
├── history.rs
├── profiler.rs
├── project.rs
├── runtime.rs
├── scene.rs
├── scene_model.rs
├── scene_tools.rs
├── selection.rs
│
├── assets/
├── docking/
├── settings/
├── ui/
├── viewport/
├── viewport2d/
└── plugins/
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
