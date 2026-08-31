use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisualNodeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VisualNodeKind {
    Start,
    Update,
    Branch,
    Sequence,
    SetPosition([f32; 3]),
    SpawnCube,
    DestroySelf,
    Print(String),
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualNode {
    pub id: VisualNodeId,
    pub name: String,
    pub kind: VisualNodeKind,
    pub position: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualLink {
    pub from: (VisualNodeId, String),
    pub to: (VisualNodeId, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualScript {
    pub version: u32,
    pub nodes: BTreeMap<VisualNodeId, VisualNode>,
    pub links: Vec<VisualLink>,
    pub next_id: u64,
}

impl VisualScript {
    pub fn new() -> Self { Self { version: 2, ..Default::default() } }

    pub fn add_node(&mut self, name: impl Into<String>, kind: VisualNodeKind, position: [f32; 2]) -> VisualNodeId {
        let id = VisualNodeId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.nodes.insert(id, VisualNode { id, name: name.into(), kind, position });
        id
    }

    pub fn connect(&mut self, from: VisualNodeId, from_socket: impl Into<String>, to: VisualNodeId, to_socket: impl Into<String>) -> Result<(), String> {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) { return Err("visual script link references a missing node".into()); }
        self.links.push(VisualLink { from: (from, from_socket.into()), to: (to, to_socket.into()) });
        Ok(())
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.nodes.values().filter(|node| matches!(node.kind, VisualNodeKind::Start)).count() != 1 {
            issues.push("visual script should have exactly one Start node".into());
        }
        for link in &self.links {
            if !self.nodes.contains_key(&link.from.0) || !self.nodes.contains_key(&link.to.0) { issues.push("visual script contains a dangling link".into()); }
        }
        issues
    }

    pub fn next_node(&self, current: VisualNodeId, socket: &str) -> Option<VisualNodeId> {
        self.links.iter().find(|link| link.from.0 == current && link.from.1 == socket).map(|link| link.to.0)
    }

    pub fn start_node(&self) -> Option<VisualNodeId> {
        self.nodes.values().find(|node| matches!(node.kind, VisualNodeKind::Start)).map(|node| node.id)
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct VisualScriptAsset(pub VisualScript);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct VisualScriptRuntime {
    pub active: bool,
    pub current: Option<VisualNodeId>,
    pub steps_this_frame: u32,
}

impl VisualScriptRuntime {
    pub fn start(&mut self, script: &VisualScript) {
        self.active = true;
        self.current = script.start_node();
        self.steps_this_frame = 0;
    }
    pub fn stop(&mut self) { self.active = false; self.current = None; self.steps_this_frame = 0; }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct VisualScriptEventQueue(pub VecDeque<(VisualNodeId, String)>);

pub struct VisualScriptingPlugin;
impl Plugin for VisualScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualScriptEventQueue>()
            .add_systems(Update, execute_visual_scripts);
    }
}

fn execute_visual_scripts(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut queue: ResMut<VisualScriptEventQueue>,
    mut query: Query<(Entity, &VisualScriptAsset, &mut VisualScriptRuntime, &Transform)>,
) {
    for (entity, asset, mut runtime, transform) in &mut query {
        if !runtime.active { runtime.start(&asset.0); }
        runtime.steps_this_frame = 0;
        let mut current = runtime.current;
        while let Some(node_id) = current {
            if runtime.steps_this_frame >= 32 { break; }
            runtime.steps_this_frame += 1;
            let Some(node) = asset.0.nodes.get(&node_id) else { runtime.stop(); break; };
            let next_socket = match &node.kind {
                VisualNodeKind::Start => "exec",
                VisualNodeKind::Update => "exec",
                VisualNodeKind::Sequence => "next",
                VisualNodeKind::Branch => "true",
                VisualNodeKind::SetPosition(position) => {
                    commands.entity(entity).insert(Transform { translation: Vec3::from_array(*position), ..*transform });
                    "exec"
                }
                VisualNodeKind::SpawnCube => {
                    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
                    let material = materials.add(StandardMaterial { base_color: Color::srgb(0.8, 0.35, 0.2), ..default() });
                    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::from_translation(transform.translation + Vec3::Y)));
                    "exec"
                }
                VisualNodeKind::DestroySelf => {
                    commands.entity(entity).despawn();
                    runtime.stop();
                    break;
                }
                VisualNodeKind::Print(message) => {
                    info!(target: "bevy_gui::visual_scripting", "{}", message);
                    queue.0.push_back((node_id, message.clone()));
                    "exec"
                }
                VisualNodeKind::End => { runtime.stop(); break; }
            };
            current = asset.0.next_node(node_id, next_socket);
            if matches!(node.kind, VisualNodeKind::Update) { break; }
        }
        runtime.current = current;
        if time.delta_secs() == 0.0 { runtime.steps_this_frame = 0; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn script_validates_start_and_links() {
        let mut script = VisualScript::new();
        let start = script.add_node("Start", VisualNodeKind::Start, [0.0, 0.0]);
        let end = script.add_node("End", VisualNodeKind::End, [100.0, 0.0]);
        assert!(script.connect(start, "exec", end, "exec").is_ok());
        assert!(script.validate().is_empty());
        assert_eq!(script.next_node(start, "exec"), Some(end));
    }
}
