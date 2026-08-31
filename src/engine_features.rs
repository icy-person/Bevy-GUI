use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Built-in engine/editor capability flags. These mirror the useful parts of
/// Bevy-focused IDE workflows without coupling the engine core to an editor UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EngineFeature {
    SceneEditor,
    EcsInspector,
    SystemGraph,
    EventMonitor,
    QueryVisualizer,
    StateEditor,
    Animation,
    VisualScripting,
    ShaderGraph,
    AssetBrowser,
    PluginBrowser,
    Terminal,
    Lsp,
    Git,
    Profiler,
}

#[derive(Resource, Debug, Clone)]
pub struct EngineFeatureRegistry {
    enabled: BTreeSet<EngineFeature>,
}

impl Default for EngineFeatureRegistry {
    fn default() -> Self {
        Self {
            enabled: [
                EngineFeature::SceneEditor,
                EngineFeature::EcsInspector,
                EngineFeature::SystemGraph,
                EngineFeature::EventMonitor,
                EngineFeature::QueryVisualizer,
                EngineFeature::StateEditor,
                EngineFeature::Animation,
                EngineFeature::VisualScripting,
                EngineFeature::ShaderGraph,
                EngineFeature::AssetBrowser,
                EngineFeature::Profiler,
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl EngineFeatureRegistry {
    pub fn enable(&mut self, feature: EngineFeature) {
        self.enabled.insert(feature);
    }

    pub fn disable(&mut self, feature: EngineFeature) {
        self.enabled.remove(&feature);
    }

    pub fn is_enabled(&self, feature: EngineFeature) -> bool {
        self.enabled.contains(&feature)
    }

    pub fn iter(&self) -> impl Iterator<Item = EngineFeature> + '_ {
        self.enabled.iter().copied()
    }
}

/// Compact runtime event history for an editor monitor or a diagnostics panel.
#[derive(Resource, Debug, Clone)]
pub struct EngineEventMonitor {
    capacity: usize,
    sequence: u64,
    events: VecDeque<EngineEventRecord>,
}

#[derive(Debug, Clone)]
pub struct EngineEventRecord {
    pub sequence: u64,
    pub frame: u64,
    pub kind: String,
    pub payload: String,
}

impl Default for EngineEventMonitor {
    fn default() -> Self {
        Self::with_capacity(512)
    }
}

impl EngineEventMonitor {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            sequence: 0,
            events: VecDeque::new(),
        }
    }

    pub fn push(&mut self, frame: u64, kind: impl Into<String>, payload: impl Into<String>) {
        self.sequence = self.sequence.saturating_add(1);
        self.events.push_back(EngineEventRecord {
            sequence: self.sequence,
            frame,
            kind: kind.into(),
            payload: payload.into(),
        });
        while self.events.len() > self.capacity {
            self.events.pop_front();
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &EngineEventRecord> {
        self.events.iter()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Lightweight metadata registry used by a system graph/query diagnostics UI.
/// The actual Bevy schedules remain the source of execution truth; this layer
/// stores user-facing labels and dependency hints emitted by engine features.
#[derive(Resource, Debug, Default, Clone)]
pub struct EngineGraphRegistry {
    systems: BTreeMap<String, EngineSystemInfo>,
}

#[derive(Debug, Clone)]
pub struct EngineSystemInfo {
    pub name: String,
    pub schedule: String,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub after: Vec<String>,
    pub before: Vec<String>,
}

impl EngineGraphRegistry {
    pub fn register(&mut self, info: EngineSystemInfo) {
        self.systems.insert(info.name.clone(), info);
    }

    pub fn get(&self, name: &str) -> Option<&EngineSystemInfo> {
        self.systems.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &EngineSystemInfo> {
        self.systems.values()
    }

    pub fn dependencies_of(&self, name: &str) -> Vec<&EngineSystemInfo> {
        let Some(target) = self.systems.get(name) else {
            return Vec::new();
        };
        target
            .after
            .iter()
            .filter_map(|dependency| self.systems.get(dependency))
            .collect()
    }
}

/// Shared engine diagnostics counters.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct EngineDiagnostics {
    pub frames: u64,
    pub entities: u32,
    pub drawables: u32,
    pub assets_loaded: u64,
    pub asset_failures: u64,
    pub commands_executed: u64,
}

pub struct EngineFeaturesPlugin;

impl Plugin for EngineFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineFeatureRegistry>()
            .init_resource::<EngineEventMonitor>()
            .init_resource::<EngineGraphRegistry>()
            .init_resource::<EngineDiagnostics>()
            .add_systems(PostUpdate, collect_engine_diagnostics);
    }
}

fn collect_engine_diagnostics(
    mut diagnostics: ResMut<EngineDiagnostics>,
    entities: Query<(), With<crate::viewport::EditorEntity>>,
    commands: Option<Res<crate::command_executor::CommandExecutionState>>,
) {
    diagnostics.frames = diagnostics.frames.saturating_add(1);
    diagnostics.entities = entities.iter().count().min(u32::MAX as usize) as u32;
    if let Some(commands) = commands {
        diagnostics.commands_executed = commands.executed;
    }
}
