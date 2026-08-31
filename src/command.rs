use bevy::prelude::*;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorCommandId(pub &'static str);

#[derive(Debug, Clone)]
pub struct EditorCommand { pub id:EditorCommandId,pub label:&'static str,pub shortcut:Option<&'static str> }

#[derive(Resource,Default)]
pub struct EditorCommandRegistry{commands:BTreeMap<EditorCommandId,EditorCommand>}
impl EditorCommandRegistry{pub fn register(&mut self,command:EditorCommand){self.commands.insert(command.id,command);}pub fn iter(&self)->impl Iterator<Item=&EditorCommand>{self.commands.values()}pub fn contains(&self,id:EditorCommandId)->bool{self.commands.contains_key(&id)}pub fn get(&self,id:EditorCommandId)->Option<&EditorCommand>{self.commands.get(&id)}}

#[derive(Resource,Default)]
pub struct EditorCommandBus{queue:VecDeque<EditorCommandId>}
impl EditorCommandBus{pub fn emit(&mut self,id:EditorCommandId){self.queue.push_back(id)}pub fn drain(&mut self)->impl Iterator<Item=EditorCommandId>+'_{std::iter::from_fn(move||self.queue.pop_front())}pub fn len(&self)->usize{self.queue.len()}pub fn is_empty(&self)->bool{self.queue.is_empty()}}

#[derive(Event,Debug,Clone,Copy,PartialEq,Eq)]
pub struct HistoryCommandEvent{pub undo:bool}

#[cfg(test)]
mod tests{use super::*;#[test]fn registry_replaces_commands_by_id(){let mut registry=EditorCommandRegistry::default();registry.register(EditorCommand{id:EditorCommandId("test"),label:"Test",shortcut:None});registry.register(EditorCommand{id:EditorCommandId("test"),label:"Replacement",shortcut:Some("F1")});assert_eq!(registry.iter().count(),1);assert_eq!(registry.get(EditorCommandId("test")).unwrap().label,"Replacement");}#[test]fn command_bus_is_fifo(){let mut bus=EditorCommandBus::default();bus.emit(EditorCommandId("first"));bus.emit(EditorCommandId("second"));assert_eq!(bus.len(),2);let ids:Vec<_>=bus.drain().collect();assert_eq!(ids,vec![EditorCommandId("first"),EditorCommandId("second")]);assert!(bus.is_empty());}}
