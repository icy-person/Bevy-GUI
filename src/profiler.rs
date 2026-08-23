use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct EditorProfiler {
    pub frame_time_ms: f32,
    pub fps: f32,
    pub min_frame_ms: f32,
    pub max_frame_ms: f32,
    pub samples: u64,
    accumulator: f32,
    window_samples: u32,
}

impl Default for EditorProfiler {
    fn default() -> Self {
        Self {
            frame_time_ms: 0.0,
            fps: 0.0,
            min_frame_ms: f32::MAX,
            max_frame_ms: 0.0,
            samples: 0,
            accumulator: 0.0,
            window_samples: 0,
        }
    }
}

pub fn install_profiler(app: &mut App) {
    app.init_resource::<EditorProfiler>()
        .add_systems(Update, sample_frame_time);
}

fn sample_frame_time(time: Res<Time>, mut profiler: ResMut<EditorProfiler>) {
    let delta = time.delta_secs().max(f32::EPSILON);
    let ms = delta * 1000.0;

    profiler.frame_time_ms = ms;
    profiler.min_frame_ms = profiler.min_frame_ms.min(ms);
    profiler.max_frame_ms = profiler.max_frame_ms.max(ms);
    profiler.samples = profiler.samples.saturating_add(1);
    profiler.accumulator += delta;
    profiler.window_samples += 1;

    if profiler.accumulator >= 0.5 {
        profiler.fps = profiler.window_samples as f32 / profiler.accumulator;
        profiler.accumulator = 0.0;
        profiler.window_samples = 0;
    }
}
