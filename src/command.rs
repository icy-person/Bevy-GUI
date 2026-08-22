use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorCommandId(pub &'static str);

#[derive(Debug, Clone)]
pub struct EditorCommand {
    pub id: EditorCommandId,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
}

#[derive(Resource, Default)]
pub struct EditorCommandRegistry {
    commands: BTreeMap<EditorCommandId, EditorCommand>,
}

impl EditorCommandRegistry {
    pub fn register(&mut self, command: EditorCommand) {
        self.commands.insert(command.id, command);
    }

    pub fn iter(&self) -> impl Iterator<Item = &EditorCommand> {
        self.commands.values()
    }
}
