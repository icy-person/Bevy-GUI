# Bevy-GUI

Bevy-GUI is a plugin-first **game-engine + visual editor platform** built on Bevy 0.19. It combines a persistent scene authoring environment with a reusable runtime foundation, physics, gameplay input, animation, shader-graph and visual-scripting services, asset tooling, diagnostics and a Bevy-native/egui editor shell.

The architecture deliberately incorporates ideas from Bevy-focused tools such as Jackdaw and BerryCode while keeping implementation owned by this repository. Jackdaw is the reference for native Bevy widgets, modular panels, reflection-friendly editing and document/ECS synchronization. BerryCode is the reference for the broader Bevy-first IDE workflow: Scene Editor, ECS Inspector, System Graph, Event Monitor, Query Visualizer, Animation, Shader Graph, Visual Scripting and development tooling. BerryCode currently targets Bevy 0.18, so it is not a direct dependency of this Bevy 0.19 project.

## Engine architecture

```text
Bevy 0.19
   │
   ├── Engine runtime
   │   ├── EngineRuntimePlugin / GameConfig
   │   ├── EngineClock / frame service
   │   ├── Avian 3D physics
   │   ├── Gameplay input actions
   │   ├── Animation runtime
   │   ├── ShaderGraph model
   │   └── VisualScripting model
   │
   ├── Authoring
   │   ├── SceneDocument
   │   ├── Mesh / primitive / material metadata
   │   ├── Prefabs
   │   ├── Selection + transform history
   │   ├── Asset database/import pipeline
   │   └── 2D/3D viewport + gizmos
   │
   ├── Editor services
   │   ├── Inspector
   │   ├── Hierarchy
   │   ├── Docking
   │   ├── Command bus
   │   ├── System graph
   │   ├── Event monitor
   │   └── Diagnostics/profiler
   │
   └── UI
       ├── Bevy Feathers / Jackdaw native widgets
       ├── egui compatibility shell
       └── dockable workspace
```

The key separation is that `EngineRuntimePlugin` can be used without the editor. `BevyGuiPlugin` layers authoring and GUI services over the same engine/runtime APIs, allowing generated projects to remain standalone.

## Current engine/editor capabilities

- Bevy 0.19 engine foundation
- Standalone `build_game_app(GameConfig)` API
- Runtime `.scene.json` loader
- Persistent scene authoring with stable IDs, hierarchy and visibility
- Primitive scene nodes: Cube, Plane, Sphere and Capsule
- Mesh asset references and material metadata
- Static/Dynamic/Kinematic collision metadata using Avian 3D
- Configurable gameplay input actions
- Animation clips, tracks, keyframes and runtime playback state
- Serializable Shader Graph model with nodes, links and validation
- Serializable Visual Scripting graph with runtime state
- 3D viewport with FreeCamera, InfiniteGrid, picking and Transform Gizmo
- 2D viewport with orthographic camera/pan/zoom and grid
- World / Local transforms and Translate / Rotate / Scale
- Multi-selection, hierarchy, parent/unparent, duplicate/delete
- Live Inspector editing and Jackdaw-native text fields
- Prefab authoring and validation helpers
- Asset browser/import database
- Command bus with Play/Pause/Stop, save, build/export and authoring commands
- Play-in-editor runtime preview with scene snapshot isolation
- Runtime frame clock and configurable frame budget
- Engine diagnostics and bounded event monitoring
- System dependency metadata registry and engine tools UI
- Live profiler with FPS/frame-time history
- Docking workspace with editor and engine diagnostic views
- Project manager and persistent settings
- Standalone release export pipeline
- Android arm64 target scaffold using Bevy GameActivity + cargo-ndk
- GitHub Actions verification/build pipeline for Linux and Android arm64

## Android

Bevy 0.19 supports Android using explicit `android-game-activity` or `android-native-activity` features. This project provides a `mobile/` target configured for arm64-v8a/GameActivity. The Android workflow builds the native library with `cargo-ndk`. citeturn753743search0turn753743search2

The engine is intentionally designed so Android consumes the runtime layer rather than the desktop editor. This keeps editor-only GUI dependencies out of the mobile game process.

## Jackdaw integration

`jackdaw_feathers` is integrated as a native Bevy widget layer. The current bridge uses Jackdaw's text-edit/event model for inspector fields while the existing egui docking shell remains available for mature desktop tooling.

## BerryCode-inspired tooling

BerryCode demonstrates a useful product boundary around Bevy: scene editing, ECS inspection, system visualization, runtime event inspection, animation, visual scripting and shader graph should be separate engine/editor services rather than one monolithic window. Bevy-GUI now exposes those services as independent modules so they can be expanded without changing the engine core.

## Source layout

```text
src/
├── app.rs
├── lib.rs
├── animation.rs
├── engine.rs
├── engine_features.rs
├── engine_runtime.rs
├── engine_tools_ui.rs
├── game.rs
├── input.rs
├── shader_graph.rs
├── visual_scripting.rs
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

mobile/
└── Cargo.toml
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
Ctrl+S             Save project + scene
Ctrl+Shift+B       Build/export project
Ctrl+O             Open scene
Ctrl+Shift+S       Save scene
```

## Validation

```bash
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

GitHub Actions runs verification/builds on pushes and pull requests. The Android arm64 workflow is also available from **Actions → android → Run workflow**.
