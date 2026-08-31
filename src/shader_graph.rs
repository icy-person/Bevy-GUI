use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShaderNodeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShaderNodeKind {
    Constant([f32; 4]),
    Time,
    Texture(String),
    Multiply,
    Add,
    Lerp,
    Normal,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderNode {
    pub id: ShaderNodeId,
    pub name: String,
    pub kind: ShaderNodeKind,
    pub position: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShaderLink {
    pub from: (ShaderNodeId, String),
    pub to: (ShaderNodeId, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShaderGraph {
    pub version: u32,
    pub nodes: BTreeMap<ShaderNodeId, ShaderNode>,
    pub links: Vec<ShaderLink>,
    pub next_id: u64,
}

impl ShaderGraph {
    pub fn new() -> Self { Self { version: 1, ..Default::default() } }
    pub fn add_node(&mut self, name: impl Into<String>, kind: ShaderNodeKind, position: [f32;2]) -> ShaderNodeId { let id=ShaderNodeId(self.next_id);self.next_id+=1;self.nodes.insert(id,ShaderNode{id,name:name.into(),kind,position});id }
    pub fn remove_node(&mut self,id:ShaderNodeId)->Option<ShaderNode>{self.links.retain(|l|l.from.0!=id&&l.to.0!=id);self.nodes.remove(&id)}
    pub fn connect(&mut self,from:ShaderNodeId,from_socket:impl Into<String>,to:ShaderNodeId,to_socket:impl Into<String>)->Result<(),String>{if !self.nodes.contains_key(&from)||!self.nodes.contains_key(&to){return Err("shader link references a missing node".into())}let link=ShaderLink{from:(from,from_socket.into()),to:(to,to_socket.into())};if !self.links.contains(&link){self.links.push(link)}Ok(())}
    pub fn validate(&self)->Vec<String>{let mut issues=Vec::new();if !self.nodes.values().any(|n|matches!(n.kind,ShaderNodeKind::Output)){issues.push("graph has no Output node".into())}for link in &self.links{if !self.nodes.contains_key(&link.from.0){issues.push(format!("missing source node {:?}",link.from.0))}if !self.nodes.contains_key(&link.to.0){issues.push(format!("missing destination node {:?}",link.to.0))}}issues}
}

#[derive(Resource,Debug,Clone,Default)]
pub struct ShaderGraphLibrary { pub graphs:BTreeMap<String,ShaderGraph> }
impl ShaderGraphLibrary { pub fn insert(&mut self,name:impl Into<String>,graph:ShaderGraph){self.graphs.insert(name.into(),graph)}pub fn get(&self,name:&str)->Option<&ShaderGraph>{self.graphs.get(name)} }

pub struct ShaderGraphPlugin;
impl Plugin for ShaderGraphPlugin {fn build(&self,app:&mut App){app.init_resource::<ShaderGraphLibrary>();}}

#[cfg(test)]
mod tests{use super::*;#[test]fn graph_connects_and_validates(){let mut g=ShaderGraph::new();let output=g.add_node("Output",ShaderNodeKind::Output,[0.0,0.0]);let value=g.add_node("Value",ShaderNodeKind::Constant([1.0,0.0,0.0,1.0]),[-200.0,0.0]);assert!(g.connect(value,"value",output,"color").is_ok());assert!(g.validate().is_empty());}}
