//! The pixel document and its pure raster operations.

use image::RgbaImage;

pub type Px = [u8; 4];

/// An opaque RGBA raster canvas. The background is always opaque white.
pub struct Document {
    pub pixels: RgbaImage,
}

impl Document {
    pub fn new(width: u32, height: u32) -> Self {
        let width = width.clamp(1, 8192);
        let height = height.clamp(1, 8192);
        let mut pixels = RgbaImage::new(width, height);
        for p in pixels.pixels_mut() {
            *p = image::Rgba([255, 255, 255, 255]);
        }
        Self { pixels }
    }

    pub fn from_rgba(pixels: RgbaImage) -> Self {
        Self { pixels }
    }

    pub fn width(&self) -> u32 {
        self.pixels.width()
    }

    pub fn height(&self) -> u32 {
        self.pixels.height()
    }

    /// Composite one pixel with straight-alpha source-over.
    fn blend_pixel(&mut self, x: i64, y: i64, src: Px) {
        if x < 0 || y < 0 || x >= i64::from(self.width()) || y >= i64::from(self.height()) {
            return;
        }
        let dst = self.pixels.get_pixel(x as u32, y as u32).0;
        let a = f32::from(src[3]) / 255.0;
        if a <= 0.0 {
            return;
        }
        let out = [
            lerp_byte(dst[0], src[0], a),
            lerp_byte(dst[1], src[1], a),
            lerp_byte(dst[2], src[2], a),
            (a * 255.0 + (1.0 - a) * f32::from(dst[3]))
                .round()
                .clamp(0.0, 255.0) as u8,
        ];
        self.pixels.put_pixel(x as u32, y as u32, image::Rgba(out));
    }

    /// Stamp a circular dab centred on canvas coordinates, with an anti-aliased rim.
    pub fn stamp_disk(&mut self, cx: f32, cy: f32, radius: f32, color: Px) {
        let r = radius.max(0.5);
        let min_x = (cx - r).floor() as i64;
        let max_x = (cx + r).ceil() as i64;
        let min_y = (cy - r).floor() as i64;
        let max_y = (cy + r).ceil() as i64;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                // Coverage: 1 inside, soft rim over ~1px, 0 outside.
                let coverage = (r - dist + 0.5).clamp(0.0, 1.0);
                if coverage <= 0.0 {
                    continue;
                }
                let mut dab = color;
                dab[3] = (f32::from(color[3]) * coverage).round() as u8;
                self.blend_pixel(x, y, dab);
            }
        }
    }

    /// Stamp round dabs along a segment, spaced for a continuous stroke.
    pub fn stroke_segment(&mut self, from: (f32, f32), to: (f32, f32), radius: f32, color: Px) {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let len = (dx * dx + dy * dy).sqrt();
        let spacing = (radius * 0.3).max(0.4);
        let steps = (len / spacing).ceil() as u32;
        if steps == 0 {
            self.stamp_disk(from.0, from.1, radius, color);
            return;
        }
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = from.0 + dx * t;
            let y = from.1 + dy * t;
            self.stamp_disk(x, y, radius, color);
        }
    }

    /// 4-way exact-match flood fill.
    pub fn flood_fill(&mut self, start_x: u32, start_y: u32, color: Px) -> bool {
        let (w, h) = (self.width(), self.height());
        if start_x >= w || start_y >= h {
            return false;
        }
        let target = self.pixels.get_pixel(start_x, start_y).0;
        if target == color {
            return false;
        }
        let mut stack = vec![(start_x, start_y)];
        while let Some((x, y)) = stack.pop() {
            if x >= w || y >= h {
                continue;
            }
            if self.pixels.get_pixel(x, y).0 != target {
                continue;
            }
            self.pixels.put_pixel(x, y, image::Rgba(color));
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x + 1 < w {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y + 1 < h {
                stack.push((x, y + 1));
            }
        }
        true
    }

    /// Draw an outlined (or filled) rectangle.
    pub fn draw_rect(
        &mut self,
        a: (f32, f32),
        b: (f32, f32),
        radius: f32,
        color: Px,
        filled: bool,
    ) {
        let (x0, y0) = (a.0.min(b.0), a.1.min(b.1));
        let (x1, y1) = (a.0.max(b.0), a.1.max(b.1));
        if filled {
            self.fill_rect(x0, y0, x1, y1, color);
        } else {
            let r = radius.max(0.5);
            self.stroke_segment((x0, y0), (x1, y0), r, color);
            self.stroke_segment((x1, y0), (x1, y1), r, color);
            self.stroke_segment((x1, y1), (x0, y1), r, color);
            self.stroke_segment((x0, y1), (x0, y0), r, color);
        }
    }

    fn fill_rect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Px) {
        for y in y0.floor() as i64..=y1.ceil() as i64 {
            for x in x0.floor() as i64..=x1.ceil() as i64 {
                let cx = x as f32 + 0.5;
                let cy = y as f32 + 0.5;
                if cx >= x0 && cx <= x1 && cy >= y0 && cy <= y1 {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    /// Draw an outlined (or filled) ellipse inscribed in the given bounds.
    pub fn draw_ellipse(
        &mut self,
        a: (f32, f32),
        b: (f32, f32),
        radius: f32,
        color: Px,
        filled: bool,
    ) {
        let cx = (a.0 + b.0) / 2.0;
        let cy = (a.1 + b.1) / 2.0;
        let rx = ((a.0 - b.0).abs() / 2.0).max(0.5);
        let ry = ((a.1 - b.1).abs() / 2.0).max(0.5);
        let steps = ((rx + ry) * 2.0).ceil().max(24.0) as u32;
        if filled {
            let min_y = (cy - ry).floor() as i64;
            let max_y = (cy + ry).ceil() as i64;
            let min_x = (cx - rx).floor() as i64;
            let max_x = (cx + rx).ceil() as i64;
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let nx = (x as f32 + 0.5 - cx) / rx;
                    let ny = (y as f32 + 0.5 - cy) / ry;
                    if nx * nx + ny * ny <= 1.0 {
                        self.blend_pixel(x, y, color);
                    }
                }
            }
        } else {
            let r = radius.max(0.5);
            let mut prev = point_on_ellipse(cx, cy, rx, ry, 0.0);
            for i in 1..=steps {
                let t = std::f32::consts::TAU * i as f32 / steps as f32;
                let p = point_on_ellipse(cx, cy, rx, ry, t);
                self.stroke_segment(prev, p, r, color);
                prev = p;
            }
        }
    }

    /// Blit straight-alpha RGBA rows (used by the text tool).
    pub fn blit_rgba(&mut self, dst_x: i64, dst_y: i64, src: &[u8], src_w: usize, src_h: usize) {
        for sy in 0..src_h {
            for sx in 0..src_w {
                let i = (sy * src_w + sx) * 4;
                let px = [src[i], src[i + 1], src[i + 2], src[i + 3]];
                self.blend_pixel(dst_x + sx as i64, dst_y + sy as i64, px);
            }
        }
    }
}

fn point_on_ellipse(cx: f32, cy: f32, rx: f32, ry: f32, t: f32) -> (f32, f32) {
    (cx + rx * t.cos(), cy + ry * t.sin())
}

fn lerp_byte(dst: u8, src: u8, a: f32) -> u8 {
    (f32::from(dst) * (1.0 - a) + f32::from(src) * a).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_document_is_white() {
        let doc = Document::new(4, 3);
        assert_eq!((doc.width(), doc.height()), (4, 3));
        assert_eq!(doc.pixels.get_pixel(0, 0).0, [255, 255, 255, 255]);
    }

    #[test]
    fn stamp_paints_center() {
        let mut doc = Document::new(20, 20);
        doc.stamp_disk(10.0, 10.0, 3.0, [255, 0, 0, 255]);
        assert_eq!(doc.pixels.get_pixel(10, 10).0[0], 255);
        assert_eq!(doc.pixels.get_pixel(10, 10).0[1], 0);
        assert_eq!(doc.pixels.get_pixel(0, 0).0, [255, 255, 255, 255]);
    }

    #[test]
    fn flood_fill_replaces_region() {
        let mut doc = Document::new(10, 10);
        // Divider column.
        for y in 0..10 {
            doc.pixels.put_pixel(5, y, image::Rgba([0, 0, 0, 255]));
        }
        assert!(doc.flood_fill(0, 0, [255, 0, 0, 255]));
        assert_eq!(doc.pixels.get_pixel(0, 5).0, [255, 0, 0, 255]);
        assert_eq!(doc.pixels.get_pixel(4, 9).0, [255, 0, 0, 255]);
        assert_eq!(doc.pixels.get_pixel(6, 0).0, [255, 255, 255, 255]);
        // Same colour: no-op.
        assert!(!doc.flood_fill(0, 0, [255, 0, 0, 255]));
    }

    #[test]
    fn stroke_connects_points() {
        let mut doc = Document::new(50, 10);
        doc.stroke_segment((2.0, 5.0), (47.0, 5.0), 2.0, [0, 0, 255, 255]);
        assert_eq!(doc.pixels.get_pixel(25, 5).0[2], 255);
        assert_eq!(doc.pixels.get_pixel(25, 5).0[0], 0);
    }

    #[test]
    fn draw_rect_outline_hits_edges_only() {
        let mut doc = Document::new(20, 20);
        doc.draw_rect((2.0, 2.0), (17.0, 17.0), 1.0, [0, 0, 0, 255], false);
        assert_eq!(doc.pixels.get_pixel(10, 2).0, [0, 0, 0, 255]);
        assert_eq!(doc.pixels.get_pixel(10, 10).0, [255, 255, 255, 255]);
    }
}
