use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShaderNodeId(pub u64);
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShaderNodeKind { Constant([f32; 4]), Time, Texture(String), Multiply, Add, Lerp, Normal, Output }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderNode { pub id: ShaderNodeId, pub name: String, pub kind: ShaderNodeKind, pub position: [f32; 2] }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShaderLink { pub from: (ShaderNodeId, String), pub to: (ShaderNodeId, String) }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShaderGraph { pub version: u32, pub nodes: BTreeMap<ShaderNodeId, ShaderNode>, pub links: Vec<ShaderLink>, pub next_id: u64 }

impl ShaderGraph {
    pub fn new() -> Self { Self { version: 2, next_id: 1, ..Default::default() } }
    pub fn add_node(&mut self, name: impl Into<String>, kind: ShaderNodeKind, position: [f32; 2]) -> ShaderNodeId {
        let id = ShaderNodeId(self.next_id); self.next_id = self.next_id.saturating_add(1);
        self.nodes.insert(id, ShaderNode { id, name: name.into(), kind, position }); id
    }
    pub fn remove_node(&mut self, id: ShaderNodeId) -> Option<ShaderNode> { self.links.retain(|link| link.from.0 != id && link.to.0 != id); self.nodes.remove(&id) }
    pub fn connect(&mut self, from: ShaderNodeId, from_socket: impl Into<String>, to: ShaderNodeId, to_socket: impl Into<String>) -> Result<(), String> {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) { return Err("shader link references a missing node".into()); }
        let link = ShaderLink { from: (from, from_socket.into()), to: (to, to_socket.into()) };
        if !self.links.contains(&link) { self.links.push(link); }
        Ok(())
    }
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let outputs = self.nodes.values().filter(|node| matches!(node.kind, ShaderNodeKind::Output)).count();
        if outputs == 0 { issues.push("graph has no Output node".into()); }
        if outputs > 1 { issues.push("graph has multiple Output nodes".into()); }
        let mut incoming = BTreeSet::new();
        for link in &self.links {
            if !self.nodes.contains_key(&link.from.0) { issues.push(format!("missing source node {:?}", link.from.0)); }
            if !self.nodes.contains_key(&link.to.0) { issues.push(format!("missing destination node {:?}", link.to.0)); }
            if !incoming.insert(link.to.clone()) { issues.push(format!("multiple drivers for {:?}", link.to)); }
        }
        issues.extend(self.cycle_issues()); issues
    }
    fn cycle_issues(&self) -> Vec<String> {
        let mut edges: BTreeMap<ShaderNodeId, Vec<ShaderNodeId>> = BTreeMap::new();
        for link in &self.links { edges.entry(link.from.0).or_default().push(link.to.0); }
        let mut visiting = BTreeSet::new(); let mut visited = BTreeSet::new();
        fn visit(node: ShaderNodeId, edges: &BTreeMap<ShaderNodeId, Vec<ShaderNodeId>>, visiting: &mut BTreeSet<ShaderNodeId>, visited: &mut BTreeSet<ShaderNodeId>) -> bool {
            if visiting.contains(&node) { return true; } if visited.contains(&node) { return false; }
            visiting.insert(node); let cycle = edges.get(&node).into_iter().flatten().any(|next| visit(*next, edges, visiting, visited));
            visiting.remove(&node); visited.insert(node); cycle
        }
        if self.nodes.keys().copied().any(|node| visit(node, &edges, &mut visiting, &mut visited)) { vec!["graph contains a cycle".into()] } else { Vec::new() }
    }
    pub fn topological_order(&self) -> Result<Vec<ShaderNodeId>, String> {
        let mut indegree: BTreeMap<ShaderNodeId, usize> = self.nodes.keys().map(|id| (*id, 0)).collect();
        let mut edges: BTreeMap<ShaderNodeId, Vec<ShaderNodeId>> = BTreeMap::new();
        for link in &self.links { *indegree.entry(link.to.0).or_default() += 1; edges.entry(link.from.0).or_default().push(link.to.0); }
        let mut ready: BTreeSet<ShaderNodeId> = indegree.iter().filter_map(|(id, count)| (*count == 0).then_some(*id)).collect();
        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(id) = ready.pop_first() {
            order.push(id);
            for next in edges.get(&id).into_iter().flatten() {
                let count = indegree.get_mut(next).expect("edge points to known node"); *count -= 1;
                if *count == 0 { ready.insert(*next); }
            }
        }
        if order.len() == self.nodes.len() { Ok(order) } else { Err("cannot topologically order cyclic shader graph".into()) }
    }
    pub fn output_node(&self) -> Option<&ShaderNode> { self.nodes.values().find(|node| matches!(node.kind, ShaderNodeKind::Output)) }
    pub fn compile_wgsl(&self) -> Result<String, String> {
        let issues = self.validate(); if !issues.is_empty() { return Err(issues.join("; ")); }
        let order = self.topological_order()?; let mut variables = BTreeMap::<ShaderNodeId, String>::new(); let mut body = String::new();
        for id in order {
            let node = self.nodes.get(&id).ok_or_else(|| format!("missing node {:?}", id))?;
            let input = |socket: &str| self.links.iter().find(|link| link.to.0 == id && link.to.1 == socket).and_then(|link| variables.get(&link.from.0).cloned());
            let expr = match &node.kind {
                ShaderNodeKind::Constant(v) => format!("vec4<f32>({:.8}, {:.8}, {:.8}, {:.8})", v[0], v[1], v[2], v[3]),
                ShaderNodeKind::Normal => "vec4<f32>(0.0, 0.0, 1.0, 1.0)".into(),
                ShaderNodeKind::Add => format!("({} + {})", input("a").ok_or_else(|| format!("node {} missing input a", node.name))?, input("b").ok_or_else(|| format!("node {} missing input b", node.name))?),
                ShaderNodeKind::Multiply => format!("({} * {})", input("a").ok_or_else(|| format!("node {} missing input a", node.name))?, input("b").ok_or_else(|| format!("node {} missing input b", node.name))?),
                ShaderNodeKind::Lerp => format!("mix({}, {}, {})", input("a").ok_or_else(|| format!("node {} missing input a", node.name))?, input("b").ok_or_else(|| format!("node {} missing input b", node.name))?, input("t").ok_or_else(|| format!("node {} missing input t", node.name))?),
                ShaderNodeKind::Output => input("color").unwrap_or_else(|| "vec4<f32>(1.0, 1.0, 1.0, 1.0)".into()),
                ShaderNodeKind::Time => return Err(format!("node {} (Time) requires a runtime uniform backend", node.name)),
                ShaderNodeKind::Texture(path) => return Err(format!("node {} (Texture {}) requires a texture binding backend", node.name, path)),
            };
            let var = format!("n_{}", id.0); body.push_str(&format!("    let {} = {};\n", var, expr)); variables.insert(id, var);
        }
        let output = self.output_node().ok_or_else(|| "graph has no Output node".to_owned())?;
        let output_var = variables.get(&output.id).cloned().ok_or_else(|| "output node has no generated expression".to_owned())?;
        Ok(format!("@fragment\nfn fragment_main() -> @location(0) vec4<f32> {{\n{}    return {};\n}}\n", body, output_var))
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ShaderGraphLibrary { pub graphs: BTreeMap<String, ShaderGraph> }
impl ShaderGraphLibrary { pub fn insert(&mut self, name: impl Into<String>, graph: ShaderGraph) { self.graphs.insert(name.into(), graph); } pub fn get(&self, name: &str) -> Option<&ShaderGraph> { self.graphs.get(name) } }
pub struct ShaderGraphPlugin;
impl Plugin for ShaderGraphPlugin { fn build(&self, app: &mut App) { app.init_resource::<ShaderGraphLibrary>(); } }

#[cfg(test)]
mod tests { use super::*; #[test] fn graph_compiles_constant_output() { let mut graph = ShaderGraph::new(); let value = graph.add_node("Value", ShaderNodeKind::Constant([1.0,0.0,0.0,1.0]), [-200.0,0.0]); let output = graph.add_node("Output", ShaderNodeKind::Output, [0.0,0.0]); graph.connect(value,"value",output,"color").unwrap(); let wgsl = graph.compile_wgsl().unwrap(); assert!(wgsl.contains("@fragment")); assert!(wgsl.contains("return n_2")); } }
