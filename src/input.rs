use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EngineAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Primary,
    Secondary,
}

#[derive(Resource, Debug, Clone)]
pub struct EngineInputMap {
    pub bindings: BTreeMap<EngineAction, KeyCode>,
}

impl Default for EngineInputMap {
    fn default() -> Self {
        Self {
            bindings: BTreeMap::from([
                (EngineAction::MoveForward, KeyCode::KeyW),
                (EngineAction::MoveBackward, KeyCode::KeyS),
                (EngineAction::MoveLeft, KeyCode::KeyA),
                (EngineAction::MoveRight, KeyCode::KeyD),
                (EngineAction::Jump, KeyCode::Space),
                (EngineAction::Primary, KeyCode::KeyE),
                (EngineAction::Secondary, KeyCode::KeyQ),
            ]),
        }
    }
}

impl EngineInputMap {
    pub fn bind(&mut self, action: EngineAction, key: KeyCode) {
        self.bindings.insert(action, key);
    }

    pub fn key(&self, action: EngineAction) -> Option<KeyCode> {
        self.bindings.get(&action).copied()
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct EngineActionEvent {
    pub action: EngineAction,
    pub just_pressed: bool,
    pub pressed: bool,
}

pub struct EngineInputPlugin;

impl Plugin for EngineInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineInputMap>()
            .add_message::<EngineActionEvent>()
            .add_systems(Update, emit_engine_actions);
    }
}

fn emit_engine_actions(
    input: Res<ButtonInput<KeyCode>>,
    map: Res<EngineInputMap>,
    mut events: MessageWriter<EngineActionEvent>,
) {
    for (&action, &key) in &map.bindings {
        let just_pressed = input.just_pressed(key);
        let pressed = input.pressed(key);
        if just_pressed || pressed {
            events.write(EngineActionEvent {
                action,
                just_pressed,
                pressed,
            });
        }
    }
}
