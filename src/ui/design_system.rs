use bevy_egui::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis { Primary, Secondary, Danger, Warning, Success, Info }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSize { Small, Medium, Large }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface { Base, Container, High, Highest }
pub const XS:f32=4.0; pub const SM:f32=8.0; pub const MD:f32=12.0; pub const LG:f32=16.0; pub const XL:f32=24.0;
pub fn primary()->egui::Color32{egui::Color32::from_rgb(207,188,255)}
pub fn text()->egui::Color32{egui::Color32::from_rgb(232,227,239)}
pub fn secondary()->egui::Color32{egui::Color32::from_rgb(198,192,206)}
pub fn disabled()->egui::Color32{egui::Color32::from_rgb(120,119,128)}
pub fn background()->egui::Color32{egui::Color32::from_rgb(15,16,20)}
pub fn container()->egui::Color32{egui::Color32::from_rgb(23,24,29)}
pub fn high()->egui::Color32{egui::Color32::from_rgb(32,33,40)}
pub fn highest()->egui::Color32{egui::Color32::from_rgb(42,43,51)}
pub fn outline()->egui::Color32{egui::Color32::from_rgb(74,75,86)}
pub fn error()->egui::Color32{egui::Color32::from_rgb(255,180,171)}
pub fn warning_color()->egui::Color32{egui::Color32::from_rgb(255,190,96)}
pub fn success_color()->egui::Color32{egui::Color32::from_rgb(125,220,160)}
pub fn info_color()->egui::Color32{egui::Color32::from_rgb(135,200,255)}
pub fn color(e:Emphasis)->egui::Color32{match e{Emphasis::Primary=>primary(),Emphasis::Secondary=>secondary(),Emphasis::Danger=>error(),Emphasis::Warning=>warning_color(),Emphasis::Success=>success_color(),Emphasis::Info=>info_color()}}
pub fn surface_color(s:Surface)->egui::Color32{match s{Surface::Base=>background(),Surface::Container=>container(),Surface::High=>high(),Surface::Highest=>highest()}}
pub fn height(s:ControlSize)->f32{match s{ControlSize::Small=>28.0,ControlSize::Medium=>36.0,ControlSize::Large=>44.0}}
pub fn radius(s:ControlSize)->u8{match s{ControlSize::Small=>7,ControlSize::Medium=>9,ControlSize::Large=>12}}
pub fn text_button(ui:&mut egui::Ui,label:&str,size:ControlSize)->egui::Response{ui.add_sized([label.len() as f32*7.0+24.0,height(size)],egui::Button::new(label).corner_radius(egui::CornerRadius::same(radius(size))))}
pub fn primary_button(ui:&mut egui::Ui,label:&str)->egui::Response{ui.add_sized([label.len() as f32*7.0+26.0,36.0],egui::Button::new(label).fill(primary()).corner_radius(egui::CornerRadius::same(9)))}
pub fn outline_button(ui:&mut egui::Ui,label:&str)->egui::Response{ui.add_sized([label.len() as f32*7.0+26.0,36.0],egui::Button::new(label).stroke(egui::Stroke::new(1.0,outline())).corner_radius(egui::CornerRadius::same(9)))}
pub fn icon_button(ui:&mut egui::Ui,icon:&str,tip:&str,active:bool)->egui::Response{let fill=if active{primary().gamma_multiply(0.18)}else{egui::Color32::TRANSPARENT};ui.add_sized([40.0,36.0],egui::Button::new(icon).fill(fill).corner_radius(egui::CornerRadius::same(10))).on_hover_text(tip)}
pub fn toolbar_button(ui:&mut egui::Ui,label:&str,active:bool)->egui::Response{let fill=if active{primary().gamma_multiply(0.16)}else{egui::Color32::TRANSPARENT};ui.add_sized([42.0,34.0],egui::Button::new(label).fill(fill).corner_radius(egui::CornerRadius::same(8)))}
pub fn toolbar_text_button(ui:&mut egui::Ui,label:&str)->egui::Response{text_button(ui,label,ControlSize::Medium)}
pub fn toolbar_icon(ui:&mut egui::Ui,icon:&str,tip:&str)->egui::Response{icon_button(ui,icon,tip,false)}
pub fn material_surface(ui:&mut egui::Ui,body_fn:impl FnOnce(&mut egui::Ui)){egui::Frame::new().fill(background()).show(ui,body_fn)}
pub fn card(ui:&mut egui::Ui,level:Surface,body_fn:impl FnOnce(&mut egui::Ui)){egui::Frame::new().fill(surface_color(level)).stroke(egui::Stroke::new(1.0,outline().gamma_multiply(0.55))).corner_radius(egui::CornerRadius::same(12)).inner_margin(egui::Margin::same(MD as i8)).show(ui,body_fn)}
pub fn section(ui:&mut egui::Ui,title:&str,body_fn:impl FnOnce(&mut egui::Ui)){card(ui,Surface::Container,|ui|{ui.label(egui::RichText::new(title).size(15.0).strong());ui.add_space(SM);body_fn(ui)})}
pub fn separator(ui:&mut egui::Ui){ui.add_space(2.0);ui.separator();ui.add_space(2.0)}
pub fn spacer(ui:&mut egui::Ui,value:f32){ui.add_space(value)}
pub fn heading(ui:&mut egui::Ui,title:&str){ui.label(egui::RichText::new(title).size(18.0).strong().color(text()))}
pub fn subheading(ui:&mut egui::Ui,title:&str){ui.label(egui::RichText::new(title).size(14.0).strong().color(text()))}
pub fn body(ui:&mut egui::Ui,value:&str){ui.label(egui::RichText::new(value).size(13.0).color(text()))}
pub fn caption(ui:&mut egui::Ui,value:&str){ui.label(egui::RichText::new(value).size(11.0).color(secondary()))}
pub fn mono(ui:&mut egui::Ui,value:&str){ui.monospace(egui::RichText::new(value).size(11.0).color(secondary()))}
pub fn status(ui:&mut egui::Ui,value:&str,e:Emphasis){ui.label(egui::RichText::new(value).size(11.0).color(color(e)))}
pub fn status_dot(ui:&mut egui::Ui,active:bool,label_text:&str){ui.horizontal(|ui|{ui.label(egui::RichText::new("●").color(if active{success_color()}else{disabled()}));caption(ui,label_text)})}
pub fn row(ui:&mut egui::Ui,body_fn:impl FnOnce(&mut egui::Ui)){ui.horizontal(body_fn)}
pub fn column(ui:&mut egui::Ui,body_fn:impl FnOnce(&mut egui::Ui)){ui.vertical(body_fn)}
pub fn right(ui:&mut egui::Ui,body_fn:impl FnOnce(&mut egui::Ui)){ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),body_fn)}
pub fn center(ui:&mut egui::Ui,body_fn:impl FnOnce(&mut egui::Ui)){ui.horizontal_centered(body_fn)}
pub fn label_row(ui:&mut egui::Ui,label:&str,value:&str){row(ui,|ui|{caption(ui,label);right(ui,|ui|body(ui,value))})}
pub fn text_row(ui:&mut egui::Ui,label:&str,value:&mut String)->bool{let mut changed=false;row(ui,|ui|{caption(ui,label);changed=ui.text_edit_singleline(value).changed()});changed}
pub fn bool_row(ui:&mut egui::Ui,label:&str,value:&mut bool)->bool{ui.checkbox(value,label).changed()}
pub fn slider_row(ui:&mut egui::Ui,label:&str,value:&mut f32,range:std::ops::RangeInclusive<f32>)->bool{let mut changed=false;row(ui,|ui|{caption(ui,label);changed=ui.add(egui::Slider::new(value,range)).changed()});changed}
pub fn drag_row(ui:&mut egui::Ui,label:&str,value:&mut f32)->bool{let mut changed=false;row(ui,|ui|{caption(ui,label);changed=ui.add(egui::DragValue::new(value).speed(0.05)).changed()});changed}
pub fn color_row(ui:&mut egui::Ui,label:&str,value:&mut egui::Color32)->bool{let mut changed=false;row(ui,|ui|{caption(ui,label);changed=ui.color_edit_button_srgba(value).changed()});changed}
pub fn search(ui:&mut egui::Ui,query:&mut String,hint:&str)->bool{ui.add(egui::TextEdit::singleline(query).hint_text(hint).desired_width(ui.available_width())).changed()}
pub fn checkbox(ui:&mut egui::Ui,label:&str,value:&mut bool)->bool{bool_row(ui,label,value)}
pub fn empty(ui:&mut egui::Ui,icon:&str,title:&str,description:&str){center(ui,|ui|{column(ui,|ui|{ui.label(egui::RichText::new(icon).size(28.0));heading(ui,title);caption(ui,description)})})}
pub fn info(ui:&mut egui::Ui,message:&str){card(ui,Surface::High,|ui|status(ui,message,Emphasis::Info))}
pub fn warning(ui:&mut egui::Ui,message:&str){card(ui,Surface::High,|ui|status(ui,message,Emphasis::Warning))}
pub fn danger(ui:&mut egui::Ui,message:&str){card(ui,Surface::High,|ui|status(ui,message,Emphasis::Danger))}
pub fn success(ui:&mut egui::Ui,message:&str){card(ui,Surface::High,|ui|status(ui,message,Emphasis::Success))}
pub fn progress(ui:&mut egui::Ui,value:f32,text_value:&str){ui.add(egui::ProgressBar::new(value.clamp(0.0,1.0)).text(text_value))}
pub fn metric(ui:&mut egui::Ui,label:&str,value:&str,e:Emphasis){card(ui,Surface::Container,|ui|{caption(ui,label);ui.label(egui::RichText::new(value).size(20.0).strong().color(color(e)))})}
pub fn badge(ui:&mut egui::Ui,value:&str,e:Emphasis){ui.label(egui::RichText::new(value).size(10.0).color(color(e)))}
pub fn dirty_badge(ui:&mut egui::Ui,dirty:bool){if dirty{badge(ui,"● Unsaved",Emphasis::Warning)}else{badge(ui,"Saved",Emphasis::Success)}}
pub fn tree_row(ui:&mut egui::Ui,depth:usize,icon:&str,name:&str,selected:bool)->egui::Response{ui.horizontal(|ui|{ui.add_space(depth as f32*14.0);ui.selectable_label(selected,format!("{icon} {name}"))}).inner}
pub fn tree_toggle(ui:&mut egui::Ui,open:&mut bool){if ui.small_button(if *open{"▾"}else{"▸"}).clicked(){*open=!*open}}
pub fn list_row(ui:&mut egui::Ui,icon:&str,name:&str,meta:&str,selected:bool)->bool{ui.selectable_label(selected,format!("{icon} {name} — {meta}")).clicked()}
pub fn key_hint(ui:&mut egui::Ui,key:&str){ui.monospace(key)}
pub fn key_help(ui:&mut egui::Ui,label:&str,key:&str){row(ui,|ui|{body(ui,label);right(ui,|ui|key_hint(ui,key))})}
pub fn toolbar(ui:&mut egui::Ui,body_fn:impl FnOnce(&mut egui::Ui)){egui::Frame::new().fill(surface_color(Surface::Container)).corner_radius(egui::CornerRadius::same(9)).inner_margin(egui::Margin::symmetric(6,4)).show(ui,body_fn)}
pub fn toolbar_separator(ui:&mut egui::Ui){ui.separator()} pub fn toolbar_space(ui:&mut egui::Ui){ui.add_space(SM)}
pub fn play(ui:&mut egui::Ui,active:bool)->bool{toolbar_button(ui,"▶",active).clicked()} pub fn pause(ui:&mut egui::Ui,active:bool)->bool{toolbar_button(ui,"Ⅱ",active).clicked()} pub fn stop(ui:&mut egui::Ui)->bool{toolbar_button(ui,"■",false).clicked()}
pub fn scene_mode(ui:&mut egui::Ui,three_d:bool)->bool{toolbar_button(ui,"3D",three_d).clicked()} pub fn scene_mode_2d(ui:&mut egui::Ui,two_d:bool)->bool{toolbar_button(ui,"2D",two_d).clicked()}
pub fn tool_select(ui:&mut egui::Ui,active:bool)->bool{toolbar_button(ui,"↖",active).clicked()} pub fn tool_move(ui:&mut egui::Ui,active:bool)->bool{toolbar_button(ui,"↕",active).clicked()} pub fn tool_rotate(ui:&mut egui::Ui,active:bool)->bool{toolbar_button(ui,"⟳",active).clicked()} pub fn tool_scale(ui:&mut egui::Ui,active:bool)->bool{toolbar_button(ui,"↔",active).clicked()} pub fn tool_snap(ui:&mut egui::Ui,active:bool)->bool{toolbar_button(ui,"Snap",active).clicked()} pub fn tool_space(ui:&mut egui::Ui,world:bool)->bool{toolbar_button(ui,if world{"World"}else{"Local"},world).clicked()}
pub fn heading_row(ui:&mut egui::Ui,title:&str,count:Option<usize>){row(ui,|ui|{heading(ui,title);if let Some(c)=count{badge(ui,&c.to_string(),Emphasis::Secondary)}})}
pub fn footer(ui:&mut egui::Ui,left:&str,right_text:&str){row(ui,|ui|{caption(ui,left);right(ui,|ui|caption(ui,right_text))})}
pub fn save(ui:&mut egui::Ui,dirty:bool)->bool{text_button(ui,if dirty{"Save *"}else{"Save"},ControlSize::Medium).clicked()}pub fn build(ui:&mut egui::Ui)->bool{primary_button(ui,"Build").clicked()}pub fn refresh(ui:&mut egui::Ui)->bool{text_button(ui,"Refresh",ControlSize::Small).clicked()}pub fn create(ui:&mut egui::Ui)->bool{text_button(ui,"＋ Entity",ControlSize::Small).clicked()}pub fn duplicate(ui:&mut egui::Ui)->bool{text_button(ui,"Duplicate",ControlSize::Small).clicked()}pub fn remove(ui:&mut egui::Ui)->bool{outline_button(ui,"Delete").clicked()}
pub fn combo(ui:&mut egui::Ui,label:&str,value:&mut usize,items:&[&str]){egui::ComboBox::from_id_salt(label).selected_text(items.get(*value).copied().unwrap_or("-")).show_ui(ui,|ui|{for(i,item)in items.iter().enumerate(){if ui.selectable_label(*value==i,*item).clicked(){*value=i;ui.close()}}})}
pub fn tabs(ui:&mut egui::Ui,labels:&[&str],selected:&mut usize){row(ui,|ui|{for(i,label)in labels.iter().enumerate(){if ui.selectable_label(*selected==i,*label).clicked(){*selected=i}}})}
pub fn property(ui:&mut egui::Ui,label:&str,value:&str){label_row(ui,label,value)}pub fn property_float(ui:&mut egui::Ui,label:&str,value:&mut f32)->bool{drag_row(ui,label,value)}pub fn property_bool(ui:&mut egui::Ui,label:&str,value:&mut bool)->bool{bool_row(ui,label,value)}pub fn property_string(ui:&mut egui::Ui,label:&str,value:&mut String)->bool{text_row(ui,label,value)}pub fn property_color(ui:&mut egui::Ui,label:&str,value:&mut egui::Color32)->bool{color_row(ui,label,value)}
pub fn nav_button(ui:&mut egui::Ui,icon:&str,label:&str,active:bool)->egui::Response{let fill=if active{primary().gamma_multiply(0.18)}else{egui::Color32::TRANSPARENT};ui.add_sized([118.0,34.0],egui::Button::new(format!("{icon}  {label}")).fill(fill).corner_radius(egui::CornerRadius::same(9)))}
pub fn overlay(ui:&mut egui::Ui,body_fn:impl FnOnce(&mut egui::Ui)){egui::Frame::new().fill(surface_color(Surface::Highest)).stroke(egui::Stroke::new(1.0,outline())).corner_radius(egui::CornerRadius::same(12)).inner_margin(egui::Margin::same(10)).show(ui,body_fn)}
pub fn console_info(ui:&mut egui::Ui,message:&str){ui.monospace(egui::RichText::new(format!("[info] {message}")).color(secondary()))}
pub fn command_row(ui:&mut egui::Ui,name:&str,shortcut:&str){row(ui,|ui|{body(ui,name);right(ui,|ui|key_hint(ui,shortcut))})}
pub fn shortcut_row(ui:&mut egui::Ui,action:&str,keys:&str){command_row(ui,action,keys)}
pub fn diagnostic_row(ui:&mut egui::Ui,severity:Emphasis,message:&str){row(ui,|ui|{status(ui,"●",severity);caption(ui,message)})}
pub fn section_action(ui:&mut egui::Ui,label:&str)->bool{text_button(ui,label,ControlSize::Small).clicked()}pub fn section_primary(ui:&mut egui::Ui,label:&str)->bool{primary_button(ui,label).clicked()}pub fn section_danger(ui:&mut egui::Ui,label:&str)->bool{outline_button(ui,label).clicked()}pub fn settings_reset(ui:&mut egui::Ui)->bool{outline_button(ui,"Reset").clicked()}
pub fn plugin_row(ui:&mut egui::Ui,name:&str,version:&str,enabled:bool){row(ui,|ui|{status_dot(ui,enabled,name);right(ui,|ui|caption(ui,version))})}
