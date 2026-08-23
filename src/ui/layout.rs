use bevy_egui::egui;

pub const APP_BAR_HEIGHT: f32 = 56.0;
pub const NAV_RAIL_WIDTH: f32 = 76.0;
pub const LEFT_PANEL_WIDTH: f32 = 280.0;
pub const RIGHT_PANEL_WIDTH: f32 = 320.0;
pub const BOTTOM_PANEL_HEIGHT: f32 = 240.0;
pub const GAP: f32 = 8.0;
pub const RADIUS: u8 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainPage { Home, Scene, Assets, Settings }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneMode { TwoD, ThreeD }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode { Select, Move, Rotate, Scale, Pan }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomPage { Console, Profiler, Output, Diagnostics }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftPanelPage { Hierarchy, Assets }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode { Dark, Light, System }

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub page: MainPage,
    pub scene_mode: SceneMode,
    pub tool: ToolMode,
    pub bottom_page: BottomPage,
    pub left_page: LeftPanelPage,
    pub bottom_open: bool,
    pub left_open: bool,
    pub right_open: bool,
    pub command_palette: bool,
    pub quick_search: bool,
    pub settings_dialog: bool,
    pub project_menu: bool,
    pub play_menu: bool,
    pub fullscreen: bool,
    pub snap: bool,
    pub world_space: bool,
    pub status_message: String,
}

impl Default for LayoutState {
    fn default() -> Self { Self { page: MainPage::Scene, scene_mode: SceneMode::ThreeD, tool: ToolMode::Select, bottom_page: BottomPage::Console, left_page: LeftPanelPage::Hierarchy, bottom_open: false, left_open: true, right_open: true, command_palette: false, quick_search: false, settings_dialog: false, project_menu: false, play_menu: false, fullscreen: false, snap: true, world_space: true, status_message: "Ready".into() } }
}

impl LayoutState {
    pub fn set_page(&mut self, page: MainPage) { self.close_popups(); self.page = page; }
    pub fn toggle_bottom(&mut self, page: BottomPage) { if self.bottom_page == page { self.bottom_open = !self.bottom_open; } else { self.bottom_page = page; self.bottom_open = true; } }
    pub fn toggle_left(&mut self, page: LeftPanelPage) { self.left_page = page; self.left_open = true; }
    pub fn close_popups(&mut self) { self.command_palette = false; self.quick_search = false; self.project_menu = false; self.play_menu = false; }
    pub fn reset(&mut self) { *self = Self::default(); }
}

pub fn panel(ui: &mut egui::Ui, fill: egui::Color32, body: impl FnOnce(&mut egui::Ui)) { egui::Frame::new().fill(fill).inner_margin(egui::Margin::same(12)).show(ui, body); }

pub fn card(ui: &mut egui::Ui, fill: egui::Color32, body: impl FnOnce(&mut egui::Ui)) { egui::Frame::group(ui.style()).fill(fill).corner_radius(egui::CornerRadius::same(RADIUS)).inner_margin(egui::Margin::same(12)).show(ui, body); }

pub fn toolbar_button(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response { let mut button = egui::Button::new(label).corner_radius(egui::CornerRadius::same(8)); if active { button = button.fill(ui.visuals().selection.bg_fill); } ui.add_sized([42.0, 34.0], button) }

pub fn navigation_button(ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> egui::Response { let mut button = egui::Button::new(egui::RichText::new(icon).size(19.0)).corner_radius(egui::CornerRadius::same(12)); if active { button = button.fill(ui.visuals().selection.bg_fill); } ui.add_sized([56.0, 48.0], button).on_hover_text(label) }

pub fn heading(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>) { ui.horizontal(|ui| { ui.label(egui::RichText::new(title).size(16.0).strong()); if let Some(subtitle) = subtitle { ui.label(egui::RichText::new(subtitle).weak().size(11.0)); } }); }

pub fn metric(ui: &mut egui::Ui, label: &str, value: &str) { card(ui, egui::Color32::from_rgb(32,32,39), |ui| { ui.label(egui::RichText::new(value).size(20.0).strong()); ui.label(egui::RichText::new(label).weak().size(11.0)); }); }

pub fn separator(ui: &mut egui::Ui) { ui.add_space(2.0); ui.separator(); ui.add_space(2.0); }

pub fn empty(ui: &mut egui::Ui, icon: &str, title: &str, description: &str) { ui.centered_and_justified(|ui| { ui.vertical_centered(|ui| { ui.label(egui::RichText::new(icon).size(30.0)); ui.add_space(8.0); ui.label(egui::RichText::new(title).size(18.0).strong()); ui.add_space(4.0); ui.label(egui::RichText::new(description).weak()); }); }); }

// The rest of this module deliberately owns layout behavior rather than business logic.
// Each helper is kept small so panels share the same spatial rules and do not invent
// independent geometry. The explicit functions below document the supported layout
// contract and make it straightforward to extend the editor without reintroducing the
// previous collection of overlapping panels.

pub fn top_bar_rect(rect: egui::Rect) -> egui::Rect { egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + APP_BAR_HEIGHT)) }
pub fn content_rect(rect: egui::Rect) -> egui::Rect { rect.shrink2(egui::vec2(0.0, APP_BAR_HEIGHT)) }
pub fn nav_rect(rect: egui::Rect) -> egui::Rect { egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.min.y + APP_BAR_HEIGHT), egui::pos2(rect.min.x + NAV_RAIL_WIDTH, rect.max.y)) }
pub fn left_rect(rect: egui::Rect) -> egui::Rect { egui::Rect::from_min_max(egui::pos2(rect.min.x + NAV_RAIL_WIDTH, rect.min.y), egui::pos2(rect.min.x + NAV_RAIL_WIDTH + LEFT_PANEL_WIDTH, rect.max.y)) }
pub fn right_rect(rect: egui::Rect) -> egui::Rect { egui::Rect::from_min_max(egui::pos2(rect.max.x - RIGHT_PANEL_WIDTH, rect.min.y), rect.max) }
pub fn center_rect(rect: egui::Rect) -> egui::Rect { egui::Rect::from_min_max(egui::pos2(rect.min.x + NAV_RAIL_WIDTH + LEFT_PANEL_WIDTH, rect.min.y), egui::pos2(rect.max.x - RIGHT_PANEL_WIDTH, rect.max.y)) }
pub fn bottom_rect(rect: egui::Rect) -> egui::Rect { egui::Rect::from_min_max(egui::pos2(rect.min.x + NAV_RAIL_WIDTH + LEFT_PANEL_WIDTH, rect.max.y - BOTTOM_PANEL_HEIGHT), egui::pos2(rect.max.x - RIGHT_PANEL_WIDTH, rect.max.y)) }

pub fn clamp_panel_width(value: f32, minimum: f32, maximum: f32) -> f32 { value.clamp(minimum, maximum) }
pub fn clamp_panel_height(value: f32, minimum: f32, maximum: f32) -> f32 { value.clamp(minimum, maximum) }
pub fn is_inside(rect: egui::Rect, position: egui::Pos2) -> bool { rect.contains(position) }
pub fn available_width(rect: egui::Rect) -> f32 { rect.width() }
pub fn available_height(rect: egui::Rect) -> f32 { rect.height() }
pub fn center(rect: egui::Rect) -> egui::Pos2 { rect.center() }
pub fn top_left(rect: egui::Rect) -> egui::Pos2 { rect.left_top() }
pub fn top_right(rect: egui::Rect) -> egui::Pos2 { rect.right_top() }
pub fn bottom_left(rect: egui::Rect) -> egui::Pos2 { rect.left_bottom() }
pub fn bottom_right(rect: egui::Rect) -> egui::Pos2 { rect.right_bottom() }

pub fn two_column(ui: &mut egui::Ui, left_width: f32, left: impl FnOnce(&mut egui::Ui), right: impl FnOnce(&mut egui::Ui)) { ui.columns(2, |columns| { columns[0].allocate_ui_with_layout(egui::vec2(left_width, columns[0].available_height()), egui::Layout::top_down(egui::Align::Min), left); columns[1].allocate_ui_with_layout(egui::vec2((columns[1].available_width()-left_width).max(0.0), columns[1].available_height()), egui::Layout::top_down(egui::Align::Min), right); }); }

pub fn three_column(ui: &mut egui::Ui, left: impl FnOnce(&mut egui::Ui), center: impl FnOnce(&mut egui::Ui), right: impl FnOnce(&mut egui::Ui)) { ui.columns(3, |columns| { left(&mut columns[0]); center(&mut columns[1]); right(&mut columns[2]); }); }

pub fn rows(ui: &mut egui::Ui, gap: f32, count: usize, mut row: impl FnMut(&mut egui::Ui, usize)) { for index in 0..count { row(ui, index); if index + 1 < count { ui.add_space(gap); } } }

pub fn labeled_value(ui: &mut egui::Ui, label: &str, value: impl Into<String>) { ui.horizontal(|ui| { ui.allocate_ui_with_layout(egui::vec2(100.0, ui.available_height()), egui::Layout::left_to_right(egui::Align::Center), |ui| { ui.label(egui::RichText::new(label).weak()); }); ui.label(value.into()); }); }

pub fn toggle_row(ui: &mut egui::Ui, label: &str, value: &mut bool) -> bool { let changed = ui.checkbox(value, label).changed(); if changed { ui.ctx().request_repaint(); } changed }

pub fn slider_row(ui: &mut egui::Ui, label: &str, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> bool { ui.horizontal(|ui| { ui.label(label); ui.add(egui::Slider::new(value, range)).changed() }).inner }

pub fn text_row(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool { ui.horizontal(|ui| { ui.label(label); ui.text_edit_singleline(value).changed() }).inner }

pub fn search_row(ui: &mut egui::Ui, query: &mut String, hint: &str) -> bool { ui.add(egui::TextEdit::singleline(query).hint_text(hint).desired_width(ui.available_width())).changed() }

pub fn status_pill(ui: &mut egui::Ui, text: &str, color: egui::Color32) { ui.add(egui::Label::new(egui::RichText::new(text).color(color))); }

pub fn action_row(ui: &mut egui::Ui, primary: &str, secondary: &str) -> (bool, bool) { ui.horizontal(|ui| { let first = ui.button(primary).clicked(); let second = ui.button(secondary).clicked(); (first, second) }).inner }

pub fn toolbar_group(ui: &mut egui::Ui, items: &[(&str, &str)], active: usize) -> Option<usize> { let mut clicked = None; ui.horizontal(|ui| { for (index,(icon,label)) in items.iter().enumerate() { if toolbar_button(ui, icon, index == active).on_hover_text(*label).clicked() { clicked = Some(index); } } }); clicked }

pub fn tab_bar(ui: &mut egui::Ui, labels: &[&str], active: &mut usize) { ui.horizontal(|ui| { for (index,label) in labels.iter().enumerate() { if ui.selectable_label(*active == index, *label).clicked() { *active = index; } } }); }

pub fn footer(ui: &mut egui::Ui, left: &str, right: &str) { ui.horizontal(|ui| { ui.label(egui::RichText::new(left).weak().size(11.0)); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(egui::RichText::new(right).weak().size(11.0)); }); }); }

pub fn command_hint(ui: &mut egui::Ui, key: &str, description: &str) { ui.horizontal(|ui| { ui.label(egui::RichText::new(key).monospace()); ui.label(egui::RichText::new(description).weak()); }); }

pub fn rounded_fill(color: egui::Color32, alpha: u8) -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha) }

pub fn accent_outline(color: egui::Color32) -> egui::Stroke { egui::Stroke::new(1.0, color.gamma_multiply(0.7)) }

pub fn disabled_color() -> egui::Color32 { egui::Color32::from_rgb(110,110,120) }
pub fn warning_color() -> egui::Color32 { egui::Color32::from_rgb(255,190,90) }
pub fn error_color() -> egui::Color32 { egui::Color32::from_rgb(255,120,120) }
pub fn success_color() -> egui::Color32 { egui::Color32::from_rgb(120,220,150) }
pub fn info_color() -> egui::Color32 { egui::Color32::from_rgb(125,190,255) }
