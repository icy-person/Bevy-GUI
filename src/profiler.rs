use bevy::prelude::*;
use std::collections::VecDeque;

const HISTORY_CAPACITY: usize = 240;

#[derive(Resource, Debug, Clone)]
pub struct EditorProfiler {
    pub frame_time_ms: f32,
    pub fps: f32,
    pub min_frame_ms: f32,
    pub max_frame_ms: f32,
    pub average_frame_ms: f32,
    pub one_percent_low_fps: f32,
    pub samples: u64,
    pub history_ms: VecDeque<f32>,
    pub dropped_frames: u64,
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
            average_frame_ms: 0.0,
            one_percent_low_fps: 0.0,
            samples: 0,
            history_ms: VecDeque::with_capacity(HISTORY_CAPACITY),
            dropped_frames: 0,
            accumulator: 0.0,
            window_samples: 0,
        }
    }
}

impl EditorProfiler {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn latest_history(&self) -> impl Iterator<Item = &f32> {
        self.history_ms.iter()
    }

    pub fn sample_count(&self) -> usize {
        self.history_ms.len()
    }

    pub fn frame_budget_ms(&self, target_fps: f32) -> f32 {
        if target_fps <= 0.0 {
            return f32::INFINITY;
        }
        1000.0 / target_fps
    }

    pub fn budget_percent(&self, target_fps: f32) -> f32 {
        let budget = self.frame_budget_ms(target_fps);
        if !budget.is_finite() || budget <= 0.0 {
            return 0.0;
        }
        (self.frame_time_ms / budget * 100.0).max(0.0)
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
    profiler.window_samples = profiler.window_samples.saturating_add(1);

    if ms > 33.333 {
        profiler.dropped_frames = profiler.dropped_frames.saturating_add(1);
    }

    profiler.history_ms.push_back(ms);
    while profiler.history_ms.len() > HISTORY_CAPACITY {
        profiler.history_ms.pop_front();
    }

    let history_sum: f32 = profiler.history_ms.iter().copied().sum();
    if !profiler.history_ms.is_empty() {
        profiler.average_frame_ms = history_sum / profiler.history_ms.len() as f32;
    }

    if profiler.accumulator >= 0.5 {
        profiler.fps = profiler.window_samples as f32 / profiler.accumulator;
        profiler.accumulator = 0.0;
        profiler.window_samples = 0;
    }

    let mut sorted = profiler.history_ms.iter().copied().collect::<Vec<_>>();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    if !sorted.is_empty() {
        let low_index = ((sorted.len() as f32 * 0.01).ceil() as usize).saturating_sub(1);
        let worst_frame_ms = sorted[low_index];
        profiler.one_percent_low_fps = if worst_frame_ms > 0.0 {
            1000.0 / worst_frame_ms
        } else {
            0.0
        };
    }
}
