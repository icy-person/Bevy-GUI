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
    SetPosition([f32;3]),
    SpawnCube,
    DestroySelf,
    Print(String),
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualNode { pub id:VisualNodeId,pub name:String,pub kind:VisualNodeKind,pub position:[f32;2] }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualLink { pub from:(VisualNodeId,String),pub to:(VisualNodeId,String) }
#[derive(Debug,Clone,Serialize,Deserialize,Default)]pub struct VisualScript { pub version:u32,pub nodes:BTreeMap<VisualNodeId,VisualNode>,pub links:Vec<VisualLink>,pub next_id:u64 }
impl VisualScript{
 pub fn new()->Self{Self{version:1,..default()}}
 pub fn add_node(&mut self,name:impl Into<String>,kind:VisualNodeKind,position:[f32;2])->VisualNodeId{let id=VisualNodeId(self.next_id);self.next_id+=1;self.nodes.insert(id,VisualNode{id,name:name.into(),kind,position});id}
 pub fn connect(&mut self,from:VisualNodeId,from_socket:impl Into<String>,to:VisualNodeId,to_socket:impl Into<String>)->Result<(),String>{if !self.nodes.contains_key(&from)||!self.nodes.contains_key(&to){return Err("visual script link references a missing node".into())}self.links.push(VisualLink{from:(from,from_socket.into()),to:(to,to_socket.into())});Ok(())}
 pub fn validate(&self)->Vec<String>{let mut issues=Vec::new();if self.nodes.values().filter(|n|matches!(n.kind,VisualNodeKind::Start)).count()!=1{issues.push("visual script should have exactly one Start node".into())}for link in &self.links{if !self.nodes.contains_key(&link.from.0)||!self.nodes.contains_key(&link.to.0){issues.push("visual script contains a dangling link".into())}}issues}
}

#[derive(Component,Debug,Clone,Serialize,Deserialize)]pub struct VisualScriptAsset(pub VisualScript);
#[derive(Component,Debug,Clone,Copy,Default)]pub struct VisualScriptRuntime{pub active:bool,pub current:Option<VisualNodeId>}
impl VisualScriptRuntime{pub fn start(&mut self,script:&VisualScript){self.active=true;self.current=script.nodes.values().find(|n|matches!(n.kind,VisualNodeKind::Start)).map(|n|n.id)}pub fn stop(&mut self){self.active=false;self.current=None}}

#[derive(Resource,Debug,Clone,Default)]pub struct VisualScriptEventQueue(pub VecDeque<(VisualNodeId,String)>);
pub struct VisualScriptingPlugin;
impl Plugin for VisualScriptingPlugin{fn build(&self,app:&mut App){app.init_resource::<VisualScriptEventQueue>();}}

#[cfg(test)]mod tests{use super::*;#[test]fn script_validates_start_and_links(){let mut s=VisualScript::new();let start=s.add_node("Start",VisualNodeKind::Start,[0.0,0.0]);let end=s.add_node("End",VisualNodeKind::End,[100.0,0.0]);assert!(s.connect(start,"exec",end,"exec").is_ok());assert!(s.validate().is_empty());}}
