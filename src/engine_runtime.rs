use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct EngineClock {
    pub elapsed: f64,
    pub time_scale: f32,
}
impl Default for EngineClock { fn default() -> Self { Self { elapsed: 0.0, time_scale: 1.0 } } }

#[derive(Resource, Debug, Clone, Copy)]
pub struct EngineFrameBudget { pub target_fps: f64, pub delta_seconds: f64 }
impl Default for EngineFrameBudget { fn default() -> Self { Self { target_fps: 60.0, delta_seconds: 1.0 / 60.0 } } }

#[derive(Message, Debug, Clone, Copy)]
pub struct EngineFrameEvent { pub frame: u64, pub delta_seconds: f64 }

pub struct EngineRuntimeCorePlugin;
impl Plugin for EngineRuntimeCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineClock>()
            .init_resource::<EngineFrameBudget>()
            .add_message::<EngineFrameEvent>()
            .add_systems(Update, advance_engine_clock);
    }
}

fn advance_engine_clock(
    time: Res<Time>,
    mut clock: ResMut<EngineClock>,
) {
    let scale = clock.time_scale.max(0.0);
    clock.elapsed += time.delta_secs_f64() * scale as f64;
}
