//! The top ribbon: file actions, tools, shapes, stroke size and colors.

use craft_ui::buttons::{ButtonVisuals, IconButton, IconButtonStyle};
use egui::{Color32, CornerRadius, Pos2, Rect, RichText, Sense, Stroke, Vec2};

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
    ToggleLang,
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
}

pub fn show(
    ui: &mut egui::Ui,
    l10n: &L10n,
    state: &mut RibbonState<'_>,
    lang_is_pt: bool,
) -> Vec<RibbonEvent> {
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

        let size_label = if *state.tool == Tool::Text {
            l10n.text_size_group
        } else {
            l10n.size_group
        };
        group(ui, size_label, |ui| {
            ui.add_space(2.0);
            ui.spacing_mut().slider_width = 110.0;
            ui.add(egui::Slider::new(state.stroke, 1.0..=64.0).fixed_decimals(0));
            ui.add_space(2.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
            let color = match state.active_color {
                ActiveColor::One => *state.color1,
                ActiveColor::Two => *state.color2,
            };
            let r = (*state.stroke * 0.5).clamp(1.0, 10.0);
            ui.painter().circle_filled(rect.center(), r, color);
            ui.painter()
                .circle_stroke(rect.center(), r, Stroke::new(1.0, TEXT_MUTED));
        });

        vseparator(ui);

        group(ui, l10n.colors_group, |ui| {
            color_indicator(ui, l10n, state.color1, state.color2, state.active_color);
            ui.add_space(4.0);
            palette_grid(ui, state.color1, state.color2, *state.active_color);
        });

        vseparator(ui);

        group(ui, l10n.language, |ui| {
            let pt_clicked = ui.selectable_label(lang_is_pt, "Português").clicked();
            let en_clicked = ui.selectable_label(!lang_is_pt, "English").clicked();
            if pt_clicked && !lang_is_pt || en_clicked && lang_is_pt {
                events.push(RibbonEvent::ToggleLang);
            }
        });

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
            paint_glyph(ui.painter(), rect, tint, icon)
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

/// Paint a simple monochrome glyph inside a button rect.
fn paint_glyph(painter: &egui::Painter, rect: Rect, tint: Color32, icon: Icon) {
    let c = rect.center();
    let s = rect.size().min_elem() * 0.30;
    let stroke = Stroke::new(1.7, tint);
    let thin = Stroke::new(1.3, tint);

    match icon {
        Icon::New => {
            let page = Rect::from_center_size(c, Vec2::new(s * 1.2, s * 1.5));
            painter.rect_stroke(page, 2.0, thin, egui::StrokeKind::Middle);
            painter.line_segment(
                [c + Vec2::new(-s * 0.4, 0.0), c + Vec2::new(s * 0.4, 0.0)],
                stroke,
            );
            painter.line_segment(
                [c + Vec2::new(0.0, -s * 0.4), c + Vec2::new(0.0, s * 0.4)],
                stroke,
            );
        }
        Icon::Open => {
            let back =
                Rect::from_min_size(c + Vec2::new(-s, -s * 0.4), Vec2::new(s * 2.0, s * 1.1));
            painter.rect_stroke(back, 1.0, thin, egui::StrokeKind::Middle);
            let front =
                Rect::from_min_size(c + Vec2::new(-s, -s * 0.05), Vec2::new(s * 2.0, s * 1.0));
            painter.rect_filled(front, 1.0, tint);
        }
        Icon::Save => {
            let body = Rect::from_center_size(c, Vec2::splat(s * 1.6));
            painter.rect_stroke(body, 2.0, thin, egui::StrokeKind::Middle);
            let label =
                Rect::from_min_size(c + Vec2::new(-s * 0.5, -s * 0.65), Vec2::new(s, s * 0.55));
            painter.rect_filled(label, 1.0, tint);
            let slot = Rect::from_min_size(
                c + Vec2::new(-s * 0.45, s * 0.15),
                Vec2::new(s * 0.9, s * 0.5),
            );
            painter.rect_stroke(slot, 1.0, thin, egui::StrokeKind::Middle);
        }
        Icon::SaveProj => {
            let body = Rect::from_center_size(c, Vec2::splat(s * 1.6));
            painter.rect_stroke(body, 2.0, thin, egui::StrokeKind::Middle);
            painter.line_segment(
                [c + Vec2::new(0.0, -s * 0.55), c + Vec2::new(0.0, s * 0.25)],
                stroke,
            );
            painter.line_segment(
                [
                    c + Vec2::new(-s * 0.3, -s * 0.05),
                    c + Vec2::new(0.0, s * 0.3),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    c + Vec2::new(s * 0.3, -s * 0.05),
                    c + Vec2::new(0.0, s * 0.3),
                ],
                stroke,
            );
        }
        Icon::Undo | Icon::Redo => {
            let dir = if matches!(icon, Icon::Undo) {
                -1.0
            } else {
                1.0
            };
            let points: Vec<Pos2> = (0..=18)
                .map(|i| {
                    let t =
                        std::f32::consts::PI * 0.15 + std::f32::consts::PI * 0.85 * i as f32 / 18.0;
                    c + Vec2::new(dir * s * 1.1 * t.cos(), s * 0.9 * t.sin() * 0.9)
                })
                .collect();
            for pair in points.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }
            let tip = *points.last().unwrap();
            let prev = *points.get(points.len() - 3).unwrap();
            let dirv = (tip - prev).normalized();
            let n = Vec2::new(-dirv.y, dirv.x);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    tip,
                    tip - dirv * s * 0.45 + n * s * 0.28,
                    tip - dirv * s * 0.45 - n * s * 0.28,
                ],
                tint,
                Stroke::NONE,
            ));
        }
        Icon::Pencil => {
            let a = c + Vec2::new(-s * 0.9, s * 0.9);
            let b = c + Vec2::new(s * 0.55, -s * 0.55);
            painter.line_segment([a, b], stroke);
            let tip_dir = (b - a).normalized();
            let n = Vec2::new(-tip_dir.y, tip_dir.x);
            painter.add(egui::Shape::convex_polygon(
                vec![b + tip_dir * s * 0.35, b + n * s * 0.22, b - n * s * 0.22],
                tint,
                Stroke::NONE,
            ));
        }
        Icon::Brush => {
            let a = c + Vec2::new(-s * 0.8, s * 0.8);
            let b = c + Vec2::new(s * 0.3, -s * 0.3);
            painter.line_segment([a, b], stroke);
            painter.circle_filled(c + Vec2::new(s * 0.55, -s * 0.55), s * 0.45, tint);
        }
        Icon::Eraser => {
            let block =
                Rect::from_center_size(c + Vec2::new(-s * 0.1, 0.0), Vec2::new(s * 1.2, s * 0.7));
            painter.rect_filled(block, 2.0, tint.gamma_multiply(0.35));
            painter.rect_stroke(block, 2.0, thin, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    c + Vec2::new(s * 0.45, -s * 0.35),
                    c + Vec2::new(s * 0.75, s * 0.35),
                ],
                stroke,
            );
        }
        Icon::Bucket => {
            let body = Rect::from_center_size(
                c + Vec2::new(-s * 0.1, s * 0.15),
                Vec2::new(s * 1.2, s * 1.0),
            );
            painter.rect_stroke(body, 2.0, thin, egui::StrokeKind::Middle);
            painter.line_segment(
                [
                    c + Vec2::new(-s * 0.7, -s * 0.35),
                    c + Vec2::new(-s * 0.1, -s * 0.75),
                ],
                stroke,
            );
            painter.circle_filled(c + Vec2::new(s * 0.85, s * 0.55), s * 0.28, tint);
        }
        Icon::Text => {
            painter.text(
                c,
                egui::Align2::CENTER_CENTER,
                "A",
                egui::FontId::proportional(s * 2.0),
                tint,
            );
        }
        Icon::Picker => {
            let a = c + Vec2::new(-s * 0.8, s * 0.8);
            let b = c + Vec2::new(s * 0.4, -s * 0.4);
            painter.line_segment([a, b], stroke);
            painter.circle_filled(b + Vec2::new(s * 0.25, -s * 0.25), s * 0.35, tint);
        }
        Icon::Line => {
            painter.line_segment(
                [c + Vec2::new(-s, s * 0.8), c + Vec2::new(s, -s * 0.8)],
                stroke,
            );
        }
        Icon::Rect => {
            painter.rect_stroke(
                Rect::from_center_size(c, Vec2::new(s * 1.8, s * 1.3)),
                1.0,
                stroke,
                egui::StrokeKind::Middle,
            );
        }
        Icon::Ellipse => {
            painter.add(egui::epaint::Shape::ellipse_stroke(
                c,
                Vec2::new(s * 0.95, s * 0.7),
                stroke,
            ));
        }
    }
}
