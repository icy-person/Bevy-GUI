use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KeyValue {
    Float(f32),
    Vec3([f32; 3]),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    pub time: f32,
    pub value: KeyValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationTrack {
    pub property: String,
    pub keys: Vec<Keyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<AnimationTrack>,
    pub looping: bool,
}

impl AnimationClip {
    pub fn sample(&self, property: &str, time: f32) -> Option<KeyValue> {
        let track = self.tracks.iter().find(|track| track.property == property)?;
        let mut previous = None;
        for key in &track.keys {
            if key.time > time { break; }
            previous = Some(key.value);
        }
        previous.or_else(|| track.keys.first().map(|key| key.value))
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

impl Default for AnimatorState { fn default()->Self{Self{clip:None,time:0.0,speed:1.0,playing:false}} }
impl AnimatorState { pub fn play(&mut self,name:impl Into<String>){self.clip=Some(name.into());self.time=0.0;self.playing=true}pub fn stop(&mut self){self.playing=false;self.time=0.0}pub fn advance(&mut self,delta:f32,duration:f32,looping:bool){if !self.playing{return}self.time+=(delta*self.speed).max(0.0);if self.time>duration{if looping&&duration>0.0{self.time%=duration}else{self.time=duration;self.playing=false}}}}

pub struct AnimationRuntimePlugin;
impl Plugin for AnimationRuntimePlugin {
    fn build(&self, app:&mut App) { app.add_systems(Update, advance_animators); }
}

fn advance_animators(time:Res<Time>, libraries:Query<&AnimationLibrary>, mut animators:Query<&mut AnimatorState>) {
    for mut animator in &mut animators {
        let Some(name)=animator.clip.as_ref() else {continue};
        let duration=animators_duration_placeholder(&libraries,name).unwrap_or(0.0);
        let looping=libraries.iter().filter_map(|lib|lib.0.get(name)).next().map(|c|c.looping).unwrap_or(false);
        animator.advance(time.delta_secs(),duration,looping);
    }
}
fn animators_duration_placeholder(libraries:&Query<&AnimationLibrary>,name:&str)->Option<f32>{libraries.iter().find_map(|lib|lib.0.get(name).map(|clip|clip.duration))}

#[cfg(test)]
mod tests { use super::*; #[test] fn animator_loops(){let mut a=AnimatorState::default();a.play("walk");a.advance(1.2,1.0,true);assert!((a.time-0.2).abs()<0.001);assert!(a.playing);} }
