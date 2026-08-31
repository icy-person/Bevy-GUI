use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KeyValue { Float(f32), Vec3([f32; 3]), Bool(bool) }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe { pub time: f32, pub value: KeyValue }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationTrack { pub property: String, pub keys: Vec<Keyframe> }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationClip { pub name: String, pub duration: f32, pub tracks: Vec<AnimationTrack>, pub looping: bool }

impl AnimationClip {
    pub fn sample(&self, property: &str, time: f32) -> Option<KeyValue> {
        let track = self.tracks.iter().find(|track| track.property == property)?;
        if track.keys.is_empty() { return None; }
        let mut previous = track.keys.first().map(|key| key.value);
        for key in &track.keys {
            if key.time > time { break; }
            previous = Some(key.value);
        }
        previous
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationLibrary(pub BTreeMap<String, AnimationClip>);

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AnimatorState {
    pub clip: Option<String>,
    pub time: f32,
    pub speed: f32,
    pub playing: bool,
}

impl Default for AnimatorState { fn default() -> Self { Self { clip: None, time: 0.0, speed: 1.0, playing: false } } }

impl AnimatorState {
    pub fn play(&mut self, name: impl Into<String>) { self.clip = Some(name.into()); self.time = 0.0; self.playing = true; }
    pub fn stop(&mut self) { self.playing = false; self.time = 0.0; }
    pub fn advance(&mut self, delta: f32, duration: f32, looping: bool) {
        if !self.playing { return; }
        self.time += (delta * self.speed).max(0.0);
        if self.time > duration {
            if looping && duration > 0.0 { self.time %= duration; } else { self.time = duration; self.playing = false; }
        }
    }
}

pub struct AnimationRuntimePlugin;
impl Plugin for AnimationRuntimePlugin { fn build(&self, app: &mut App) { app.add_systems(Update, advance_animators); } }

fn advance_animators(time: Res<Time>, mut query: Query<(&AnimationLibrary, &mut AnimatorState, &mut Transform)>) {
    for (library, mut animator, mut transform) in &mut query {
        let Some(name) = animator.clip.clone() else { continue; };
        let Some(clip) = library.0.get(&name) else { animator.stop(); continue; };
        animator.advance(time.delta_secs(), clip.duration, clip.looping);
        for track in &clip.tracks {
            match (track.property.as_str(), clip.sample(&track.property, animator.time)) {
                ("translation", Some(KeyValue::Vec3(value))) => transform.translation = Vec3::from_array(value),
                ("rotation", Some(KeyValue::Vec3(value))) => transform.rotation = Quat::from_euler(EulerRot::XYZ, value[0].to_radians(), value[1].to_radians(), value[2].to_radians()),
                ("scale", Some(KeyValue::Vec3(value))) => transform.scale = Vec3::from_array(value),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn animator_loops() {
        let mut animator = AnimatorState::default();
        animator.play("walk");
        animator.advance(1.2, 1.0, true);
        assert!((animator.time - 0.2).abs() < 0.001);
        assert!(animator.playing);
    }
    #[test]
    fn clip_samples_previous_key() {
        let clip = AnimationClip { name: "move".into(), duration: 2.0, looping: false, tracks: vec![AnimationTrack { property: "translation".into(), keys: vec![Keyframe { time: 0.0, value: KeyValue::Vec3([0.0, 0.0, 0.0]) }, Keyframe { time: 1.0, value: KeyValue::Vec3([2.0, 0.0, 0.0]) }] }] };
        assert_eq!(clip.sample("translation", 1.5), Some(KeyValue::Vec3([2.0, 0.0, 0.0])));
    }
}
