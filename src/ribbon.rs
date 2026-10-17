//! The top ribbon: file actions, tools, shapes, font/text size and colors.

use craft_ui::buttons::{ButtonVisuals, IconButton, IconButtonStyle};
use egui::{Color32, CornerRadius, Rect, RichText, Sense, Stroke, Vec2};

use crate::i18n::L10n;
use crate::palette::{
    self, ACCENT, ACCENT_HOVER, BORDER, HOVER_BG, PRESSED_BG, SELECTED_BG, SELECTED_BORDER, TEXT,
    TEXT_MUTED,
};
use crate::tools::Tool;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveColor {
    One,
    Two,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RibbonEvent {
    New,
    Open,
    SavePng,
    SaveProject,
    Undo,
    Redo,
    OpenSettings,
}

/// Fields the ribbon mutates in place, plus read-only flags.
pub struct RibbonState<'a> {
    pub tool: &'a mut Tool,
    pub stroke: &'a mut f32,
    pub color1: &'a mut Color32,
    pub color2: &'a mut Color32,
    pub active_color: &'a mut ActiveColor,
    pub filled: &'a mut bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub text_size: &'a mut f32,
    pub text_font: &'a mut usize,
    pub font_names: &'a [String],
    pub text_bold: &'a mut bool,
    pub text_italic: &'a mut bool,
}

#[derive(Clone, Copy)]
enum Icon {
    New,
    Open,
    Save,
    SaveProj,
    Undo,
    Redo,
    Pencil,
    Brush,
    Eraser,
    Bucket,
    Text,
    Picker,
    Line,
    Rect,
    Ellipse,
    Settings,
}

pub fn show(ui: &mut egui::Ui, l10n: &L10n, state: &mut RibbonState<'_>) -> Vec<RibbonEvent> {
    let mut events = Vec::new();

    ui.horizontal(|ui| {
        ui.add_space(8.0);

        group(ui, "", |ui| {
            let file_buttons = [
                (Icon::New, l10n.new_file, RibbonEvent::New),
                (Icon::Open, l10n.open, RibbonEvent::Open),
                (Icon::Save, l10n.save_png, RibbonEvent::SavePng),
                (Icon::SaveProj, l10n.save_project, RibbonEvent::SaveProject),
            ];
            for (icon, label, ev) in file_buttons {
                if icon_button(ui, label, icon, false, 40.0) {
                    events.push(ev);
                }
            }
            ui.add_space(4.0);
            ui.add_enabled_ui(state.can_undo, |ui| {
                if icon_button(ui, l10n.undo, Icon::Undo, false, 40.0) {
                    events.push(RibbonEvent::Undo);
                }
            });
            ui.add_enabled_ui(state.can_redo, |ui| {
                if icon_button(ui, l10n.redo, Icon::Redo, false, 40.0) {
                    events.push(RibbonEvent::Redo);
                }
            });
        });

        vseparator(ui);

        group(ui, l10n.tools_group, |ui| {
            let tools = [
                (Icon::Pencil, l10n.pencil, Tool::Pencil),
                (Icon::Brush, l10n.brush, Tool::Brush),
                (Icon::Eraser, l10n.eraser, Tool::Eraser),
                (Icon::Bucket, l10n.bucket, Tool::Bucket),
                (Icon::Text, l10n.text, Tool::Text),
                (Icon::Picker, l10n.picker, Tool::Picker),
            ];
            for (icon, label, tool) in tools {
                if icon_button(ui, label, icon, *state.tool == tool, 40.0) {
                    *state.tool = tool;
                }
            }
        });

        vseparator(ui);

        group(ui, l10n.shapes_group, |ui| {
            let shapes = [
                (Icon::Line, l10n.line, Tool::Line),
                (Icon::Rect, l10n.rect, Tool::Rect),
                (Icon::Ellipse, l10n.ellipse, Tool::Ellipse),
            ];
            for (icon, label, tool) in shapes {
                if icon_button(ui, label, icon, *state.tool == tool, 40.0) {
                    *state.tool = tool;
                }
            }
            if state.tool.is_shape() {
                ui.add_space(4.0);
                let mut filled = *state.filled;
                if ui.checkbox(&mut filled, l10n.filled).changed() {
                    *state.filled = filled;
                }
            }
        });

        vseparator(ui);

        if *state.tool == Tool::Text {
            group(ui, l10n.font_group, |ui| {
                let mut idx = *state.text_font;
                egui::ComboBox::from_id_salt("selfpaint-font-family")
                    .selected_text(state.font_names.get(idx).map(|s| s.as_str()).unwrap_or("—"))
                    .width(170.0)
                    .show_ui(ui, |ui| {
                        for (i, name) in state.font_names.iter().enumerate() {
                            ui.selectable_value(&mut idx, i, name);
                        }
                    });
                *state.text_font = idx;
                ui.add_space(4.0);
                ui.toggle_value(&mut *state.text_bold, RichText::new("B").strong())
                    .on_hover_text(l10n.bold);
                ui.toggle_value(&mut *state.text_italic, RichText::new("I").italics())
                    .on_hover_text(l10n.italic);
            });

            vseparator(ui);
        }

        let is_text = *state.tool == Tool::Text;
        let size_label = if is_text {
            l10n.text_size_group
        } else {
            l10n.size_group
        };
        group(ui, size_label, |ui| {
            ui.add_space(2.0);
            ui.spacing_mut().slider_width = 110.0;
            if is_text {
                ui.add(egui::Slider::new(state.text_size, 8.0..=256.0).fixed_decimals(0));
            } else {
                ui.add(egui::Slider::new(state.stroke, 1.0..=64.0).fixed_decimals(0));
            }
            ui.add_space(2.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
            let color = match state.active_color {
                ActiveColor::One => *state.color1,
                ActiveColor::Two => *state.color2,
            };
            if is_text {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "A",
                    egui::FontId::proportional(16.0),
                    color,
                );
            } else {
                let r = (*state.stroke * 0.5).clamp(1.0, 10.0);
                ui.painter().circle_filled(rect.center(), r, color);
                ui.painter()
                    .circle_stroke(rect.center(), r, Stroke::new(1.0, TEXT_MUTED));
            }
        });

        vseparator(ui);

        group(ui, l10n.colors_group, |ui| {
            color_indicator(ui, l10n, state.color1, state.color2, state.active_color);
            ui.add_space(4.0);
            palette_grid(ui, state.color1, state.color2, *state.active_color);
        });

        vseparator(ui);

        if icon_button(ui, l10n.settings, Icon::Settings, false, 40.0) {
            events.push(RibbonEvent::OpenSettings);
        }

        ui.add_space(8.0);
    });

    events
}

fn group(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| add(ui));
        if !label.is_empty() {
            ui.add_space(1.0);
            ui.label(RichText::new(label).size(10.0).color(TEXT_MUTED));
        }
    });
}

fn vseparator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(9.0, 52.0), Sense::hover());
    let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
    ui.painter().vline(
        rect.center().x,
        egui::Rangef::new(rect.top() + 4.0, rect.bottom() - 4.0),
        Stroke::new(1.0, color),
    );
}

fn tool_style() -> IconButtonStyle {
    IconButtonStyle {
        normal: ButtonVisuals {
            fill: Color32::TRANSPARENT,
            stroke: Stroke::NONE,
            icon: TEXT,
        },
        hovered: ButtonVisuals {
            fill: HOVER_BG,
            stroke: Stroke::NONE,
            icon: TEXT,
        },
        pressed: Some(ButtonVisuals {
            fill: PRESSED_BG,
            stroke: Stroke::NONE,
            icon: TEXT,
        }),
        selected: ButtonVisuals {
            fill: SELECTED_BG,
            stroke: Stroke::new(1.2, SELECTED_BORDER),
            icon: ACCENT,
        },
        corner_radius: CornerRadius::same(6),
        focus_stroke: Stroke::new(1.5, ACCENT_HOVER),
    }
}

fn icon_button(ui: &mut egui::Ui, label: &str, icon: Icon, selected: bool, size: f32) -> bool {
    let style = tool_style();
    IconButton::new(label, Vec2::splat(size), &style)
        .selected(selected)
        .show(ui, |ui, rect, tint| {
            paint_icon(ui, rect, tint, icon);
        })
        .on_hover_text(label)
        .clicked()
}

fn color_indicator(
    ui: &mut egui::Ui,
    l10n: &L10n,
    color1: &mut Color32,
    color2: &mut Color32,
    active: &mut ActiveColor,
) {
    let desired = Vec2::new(46.0, 40.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    let painter = ui.painter();

    let back = Rect::from_min_size(rect.min + Vec2::new(12.0, 12.0), Vec2::new(28.0, 22.0));
    let front = Rect::from_min_size(rect.min, Vec2::new(28.0, 22.0));

    // Back swatch = color 2, front = color 1 (like MS Paint).
    painter.rect_filled(back, 4.0, *color2);
    painter.rect_stroke(
        back,
        4.0,
        Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );
    painter.rect_filled(front, 4.0, *color1);
    painter.rect_stroke(
        front,
        4.0,
        Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );

    let active_rect = match active {
        ActiveColor::One => front,
        ActiveColor::Two => back,
    };
    painter.rect_stroke(
        active_rect.expand(2.0),
        5.0,
        Stroke::new(2.0, ACCENT),
        egui::StrokeKind::Outside,
    );

    let hex = |c: &Color32| format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b());
    response.clone().on_hover_text(format!(
        "{}: {}\n{}: {}\n{} → {}",
        l10n.color1,
        hex(color1),
        l10n.color2,
        hex(color2),
        l10n.color1,
        l10n.color2
    ));

    if let Some(pos) = response.interact_pointer_pos()
        && response.clicked()
    {
        *active = if back.expand(3.0).contains(pos) {
            ActiveColor::Two
        } else {
            ActiveColor::One
        };
    }
}

fn palette_grid(
    ui: &mut egui::Ui,
    color1: &mut Color32,
    color2: &mut Color32,
    active: ActiveColor,
) {
    let swatch = 16.0;
    let gap = 2.0;
    let cols = 8;
    let rows = palette::PALETTE.len();
    let size = Vec2::new(
        cols as f32 * (swatch + gap) - gap,
        rows as f32 * (swatch + gap) - gap,
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();

    for (row, colors) in palette::PALETTE.iter().enumerate() {
        for (col, color) in colors.iter().enumerate() {
            let pos =
                rect.min + Vec2::new(col as f32 * (swatch + gap), row as f32 * (swatch + gap));
            let cell = Rect::from_min_size(pos, Vec2::splat(swatch));
            let cell_resp = ui.interact(cell, ui.id().with(("swatch", row, col)), Sense::click());
            painter.rect_filled(cell, 3.0, *color);
            painter.rect_stroke(
                cell,
                3.0,
                Stroke::new(1.0, BORDER),
                egui::StrokeKind::Inside,
            );
            if cell_resp.hovered() {
                painter.rect_stroke(cell, 3.0, Stroke::new(1.5, TEXT), egui::StrokeKind::Outside);
                cell_resp.clone().on_hover_text(format!(
                    "#{:02X}{:02X}{:02X}",
                    color.r(),
                    color.g(),
                    color.b()
                ));
            }
            if cell_resp.clicked() {
                match active {
                    ActiveColor::One => *color1 = *color,
                    ActiveColor::Two => *color2 = *color,
                }
            }
            if cell_resp.secondary_clicked() {
                // Right-click sets the other color, like MS Paint.
                match active {
                    ActiveColor::One => *color2 = *color,
                    ActiveColor::Two => *color1 = *color,
                }
            }
        }
    }
}

/// Paint a Tabler icon (white strokes) tinted to the requested color.
fn paint_icon(ui: &egui::Ui, rect: Rect, tint: Color32, icon: Icon) {
    let source = match icon {
        Icon::New => egui::include_image!("../assets/icons/file-plus.svg"),
        Icon::Open => egui::include_image!("../assets/icons/folder-open.svg"),
        Icon::Save => egui::include_image!("../assets/icons/device-floppy.svg"),
        Icon::SaveProj => egui::include_image!("../assets/icons/package.svg"),
        Icon::Undo => egui::include_image!("../assets/icons/arrow-back-up.svg"),
        Icon::Redo => egui::include_image!("../assets/icons/arrow-forward-up.svg"),
        Icon::Pencil => egui::include_image!("../assets/icons/pencil.svg"),
        Icon::Brush => egui::include_image!("../assets/icons/brush.svg"),
        Icon::Eraser => egui::include_image!("../assets/icons/eraser.svg"),
        Icon::Bucket => egui::include_image!("../assets/icons/droplet.svg"),
        Icon::Text => egui::include_image!("../assets/icons/typography.svg"),
        Icon::Picker => egui::include_image!("../assets/icons/color-picker.svg"),
        Icon::Line => egui::include_image!("../assets/icons/line.svg"),
        Icon::Rect => egui::include_image!("../assets/icons/rectangle.svg"),
        Icon::Ellipse => egui::include_image!("../assets/icons/circle.svg"),
        Icon::Settings => egui::include_image!("../assets/icons/settings.svg"),
    };
    let side = rect.size().min_elem() * 0.68;
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
    egui::Image::new(source).tint(tint).paint_at(ui, icon_rect);
}
