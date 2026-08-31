use bevy::prelude::*;
use std::time::Duration;

/// Engine-level clock shared by gameplay systems and diagnostics.
#[derive(Resource, Debug, Clone)]
pub struct EngineClock {
    pub frame: u64,
    pub elapsed: Duration,
    pub delta: Duration,
    pub time_scale: f32,
}
impl Default for EngineClock { fn default()->Self{Self{frame:0,elapsed:Duration::ZERO,delta:Duration::ZERO,time_scale:1.0}} }
impl EngineClock { pub fn scaled_delta(&self)->Duration{self.delta.mul_f32(self.time_scale.max(0.0))} }

#[derive(Resource, Debug, Clone, Copy)]
pub struct EngineFrameBudget { pub target_hz:f64,pub max_delta_seconds:f32 }
impl Default for EngineFrameBudget { fn default()->Self{Self{target_hz:60.0,max_delta_seconds:0.1}} }

#[derive(Message, Debug, Clone, Copy)]
pub struct EngineFrameEvent { pub frame:u64 }

pub struct EngineRuntimeCorePlugin;
impl Plugin for EngineRuntimeCorePlugin {
    fn build(&self,app:&mut App){
        app.init_resource::<EngineClock>()
            .init_resource::<EngineFrameBudget>()
            .add_message::<EngineFrameEvent>()
            .add_systems(First,update_engine_clock);
    }
}

fn update_engine_clock(time:Res<Time>,mut clock:ResMut<EngineClock>,mut frames:MessageWriter<EngineFrameEvent>,budget:Res<EngineFrameBudget>){
    let delta=time.delta().min(Duration::from_secs_f32(budget.max_delta_seconds.max(0.001)));
    clock.frame=clock.frame.saturating_add(1);
    clock.delta=delta;
    clock.elapsed+=delta.mul_f32(clock.time_scale.max(0.0));
    frames.write(EngineFrameEvent{frame:clock.frame});
}

#[cfg(test)]
mod tests{use super::*;#[test]fn clock_defaults_are_deterministic(){let clock=EngineClock::default();assert_eq!(clock.frame,0);assert_eq!(clock.elapsed,Duration::ZERO);assert_eq!(clock.time_scale,1.0);}}
