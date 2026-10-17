//! Canvas view: sheet placement, coordinate mapping and overlay painting.

use egui::{Color32, Pos2, Rect, Stroke, Vec2};

use crate::palette::CANVAS_BG;

pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 8.0;

pub fn rgba_to_color32(p: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3])
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasView {
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}

impl CanvasView {
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Rect of the white sheet inside the available canvas area.
    pub fn sheet_rect(&self, avail: Rect, doc_w: f32, doc_h: f32) -> Rect {
        let size = Vec2::new(doc_w * self.zoom, doc_h * self.zoom);
        let center = avail.center() + self.pan;
        Rect::from_center_size(center, size)
    }

    /// Map a screen position to canvas pixel coordinates.
    pub fn to_canvas(self, sheet: Rect, pos: Pos2) -> Option<(f32, f32)> {
        if !sheet.contains(pos) {
            return None;
        }
        let rel = pos - sheet.min;
        Some((rel.x / self.zoom, rel.y / self.zoom))
    }

    /// Map a canvas position to screen, for overlays.
    pub fn to_screen(self, sheet: Rect, canvas: (f32, f32)) -> Pos2 {
        sheet.min + Vec2::new(canvas.0 * self.zoom, canvas.1 * self.zoom)
    }

    /// Zoom that fits the document in the available area (never above 100 %).
    pub fn fit_zoom(&self, avail: Rect, doc_w: f32, doc_h: f32) -> f32 {
        if doc_w <= 0.0 || doc_h <= 0.0 || avail.width() <= 0.0 || avail.height() <= 0.0 {
            return 1.0;
        }
        let z = (avail.width() / doc_w).min(avail.height() / doc_h);
        z.clamp(MIN_ZOOM, 1.0)
    }
}

/// Paint the gray workspace, the sheet shadow, the white page and its border.
pub fn paint_sheet(painter: &egui::Painter, avail: Rect, sheet: Rect) {
    painter.rect_filled(avail, 0.0, CANVAS_BG);
    let shadow = Rect::from_min_size(sheet.min + Vec2::splat(3.0), sheet.size());
    painter.rect_filled(shadow, 0.0, Color32::from_black_alpha(60));
    painter.rect_filled(sheet, 0.0, Color32::WHITE);
    painter.rect_stroke(
        sheet,
        0.0,
        Stroke::new(1.0, Color32::from_gray(0x99)),
        egui::StrokeKind::Outside,
    );
}

/// Outline showing the brush/eraser footprint at the pointer.
pub fn paint_brush_cursor(painter: &egui::Painter, pos: Pos2, radius: f32, color: Color32) {
    painter.circle_stroke(pos, radius, Stroke::new(1.2, color));
}

/// Live preview of a shape being dragged out.
pub fn paint_shape_preview(
    painter: &egui::Painter,
    sheet: Rect,
    view: &CanvasView,
    preview: &crate::tools::ShapePreview,
) {
    let start = preview.start;
    let current = preview.current;
    let shape = preview.shape;
    let thickness = preview.thickness;
    let color = rgba_to_color32(preview.color);
    let a = view.to_screen(sheet, start);
    let b = view.to_screen(sheet, current);
    let stroke = Stroke::new(thickness.max(1.0) * view.zoom, color);
    match shape {
        crate::tools::Tool::Line => {
            painter.line_segment([a, b], stroke);
        }
        crate::tools::Tool::Rect => {
            let rect = Rect::from_two_pos(a, b);
            painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Middle);
        }
        crate::tools::Tool::Ellipse => {
            let rect = Rect::from_two_pos(a, b);
            let center = rect.center();
            let radius = Vec2::new(rect.width() / 2.0, rect.height() / 2.0);
            painter.add(egui::epaint::Shape::ellipse_stroke(center, radius, stroke));
        }
        _ => {}
    }
}

/// Constrain a drag endpoint: Shift locks lines to 45° and shapes to squares.
pub fn constrain_shift(
    start: (f32, f32),
    current: (f32, f32),
    shift: bool,
    shape: crate::tools::Tool,
) -> (f32, f32) {
    if !shift {
        return current;
    }
    match shape {
        crate::tools::Tool::Line => {
            let dx = current.0 - start.0;
            let dy = current.1 - start.1;
            let angle =
                (dy.atan2(dx) / std::f32::consts::FRAC_PI_4).round() * std::f32::consts::FRAC_PI_4;
            let len = (dx * dx + dy * dy).sqrt();
            (start.0 + len * angle.cos(), start.1 + len * angle.sin())
        }
        crate::tools::Tool::Rect | crate::tools::Tool::Ellipse => {
            let dx = current.0 - start.0;
            let dy = current.1 - start.1;
            let s = dx.abs().max(dy.abs());
            (start.0 + s * dx.signum(), start.1 + s * dy.signum())
        }
        _ => current,
    }
}
