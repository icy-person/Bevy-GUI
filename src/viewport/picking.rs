use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::Editor3dCamera;
use crate::editor::{EditorUiState, ViewportMode};
use crate::settings::EditorSettingsState;

#[derive(Resource, Debug, Clone)]
pub struct ViewportCursor {
    pub world_position: Option<Vec3>,
    pub ray_origin: Option<Vec3>,
    pub ray_direction: Option<Vec3>,
    pub grid_position: Option<Vec3>,
}

impl Default for ViewportCursor {
    fn default() -> Self {
        Self {
            world_position: None,
            ray_origin: None,
            ray_direction: None,
            grid_position: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementAxis { X, Y, Z }

impl Default for PlacementAxis { fn default() -> Self { Self::Y } }

#[derive(Resource, Debug, Clone)]
pub struct PlacementSettings {
    pub enabled: bool,
    pub axis: PlacementAxis,
    pub plane_offset: f32,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

impl Default for PlacementSettings {
    fn default() -> Self {
        Self { enabled: true, axis: PlacementAxis::Y, plane_offset: 0.0, snap_to_grid: true, grid_size: 1.0 }
    }
}

pub fn update_viewport_cursor(
    editor: Res<EditorUiState>,
    settings: Res<EditorSettingsState>,
    placement: Res<PlacementSettings>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Editor3dCamera>>,
    mut cursor: ResMut<ViewportCursor>,
) {
    if editor.viewport_mode != ViewportMode::ThreeD || !placement.enabled { cursor.clear(); return; }
    let Some(screen_position) = window.cursor_position() else { cursor.clear(); return; };
    let (camera, transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(transform, screen_position) else { cursor.clear(); return; };
    cursor.ray_origin = Some(ray.origin);
    cursor.ray_direction = Some(ray.direction.as_vec3());
    let (normal, point) = placement_plane(placement.axis, placement.plane_offset);
    let denominator = ray.direction.dot(normal);
    if denominator.abs() < 1e-6 { cursor.world_position = None; cursor.grid_position = None; return; }
    let distance = (point - ray.origin).dot(normal) / denominator;
    if distance < 0.0 { cursor.world_position = None; cursor.grid_position = None; return; }
    let world = ray.origin + ray.direction * distance;
    let grid_size = if placement.grid_size > 0.0 { placement.grid_size } else { settings.settings.viewport.grid_size.max(0.001) };
    cursor.world_position = Some(world);
    cursor.grid_position = Some(if placement.snap_to_grid { snap_point(world, normal, grid_size) } else { world });
}

pub fn draw_viewport_cursor(
    editor: Res<EditorUiState>,
    placement: Res<PlacementSettings>,
    cursor: Res<ViewportCursor>,
    mut gizmos: Gizmos,
) {
    if editor.viewport_mode != ViewportMode::ThreeD || !placement.enabled { return; }
    let Some(point) = cursor.grid_position else { return; };
    let (normal, _) = placement_plane(placement.axis, placement.plane_offset);
    let rotation = Quat::from_rotation_arc(Vec3::Y, normal);
    gizmos.circle(Isometry3d::new(point + normal * 0.01, rotation), 0.18, Color::srgb(0.3, 0.8, 1.0));
    gizmos.line(point, point + axis_vector(placement.axis) * 0.75, Color::srgb(1.0, 0.3, 0.3));
    gizmos.line(point, point + secondary_axis(placement.axis) * 0.55, Color::srgb(0.3, 1.0, 0.4));
}

fn placement_plane(axis: PlacementAxis, offset: f32) -> (Vec3, Vec3) {
    match axis { PlacementAxis::X => (Vec3::X, Vec3::X * offset), PlacementAxis::Y => (Vec3::Y, Vec3::Y * offset), PlacementAxis::Z => (Vec3::Z, Vec3::Z * offset) }
}

fn axis_vector(axis: PlacementAxis) -> Vec3 { match axis { PlacementAxis::X => Vec3::X, PlacementAxis::Y => Vec3::Y, PlacementAxis::Z => Vec3::Z } }
fn secondary_axis(axis: PlacementAxis) -> Vec3 { match axis { PlacementAxis::X => Vec3::Y, PlacementAxis::Y => Vec3::X, PlacementAxis::Z => Vec3::Y } }

fn snap_point(point: Vec3, normal: Vec3, size: f32) -> Vec3 {
    let mut result = point;
    if normal.x.abs() > 0.5 { result.y = (result.y / size).round() * size; result.z = (result.z / size).round() * size; }
    else if normal.y.abs() > 0.5 { result.x = (result.x / size).round() * size; result.z = (result.z / size).round() * size; }
    else { result.x = (result.x / size).round() * size; result.y = (result.y / size).round() * size; }
    result
}

impl ViewportCursor {
    pub fn clear(&mut self) { self.world_position = None; self.ray_origin = None; self.ray_direction = None; self.grid_position = None; }
    pub fn position_or_zero(&self) -> Vec3 { self.grid_position.unwrap_or(Vec3::ZERO) }
    pub fn has_hit(&self) -> bool { self.world_position.is_some() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn y_plane_snap_preserves_height() { let point = Vec3::new(1.49, 0.0, -2.51); assert_eq!(snap_point(point, Vec3::Y, 1.0), Vec3::new(1.0, 0.0, -3.0)); }
    #[test] fn x_plane_snap_preserves_x() { let point = Vec3::new(4.25, 1.49, -2.51); assert_eq!(snap_point(point, Vec3::X, 1.0), Vec3::new(4.25, 1.0, -3.0)); }
    #[test] fn placement_axis_maps_correctly() { assert_eq!(placement_plane(PlacementAxis::X, 2.0).0, Vec3::X); assert_eq!(placement_plane(PlacementAxis::Y, 2.0).0, Vec3::Y); assert_eq!(placement_plane(PlacementAxis::Z, 2.0).0, Vec3::Z); }
}
