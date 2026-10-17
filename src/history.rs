//! Undo/redo history based on bounding-box pixel diffs.

use image::RgbaImage;

/// One committed change: the pixels of the tightest rect that differs.
struct Change {
    x: u32,
    y: u32,
    before: RgbaImage,
    after: RgbaImage,
}

pub struct History {
    undo: Vec<Change>,
    redo: Vec<Change>,
    max_entries: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_entries: 100,
        }
    }
}

impl History {
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Record a transition between two full snapshots of the same size.
    /// Empty diffs are ignored.
    pub fn push(&mut self, before: &RgbaImage, after: &RgbaImage) {
        debug_assert_eq!(before.dimensions(), after.dimensions());
        let Some((x, y, w, h)) = diff_bbox(before, after) else {
            return;
        };
        self.redo.clear();
        self.undo.push(Change {
            x,
            y,
            before: crop(before, x, y, w, h),
            after: crop(after, x, y, w, h),
        });
        if self.undo.len() > self.max_entries {
            self.undo.remove(0);
        }
    }

    pub fn undo(&mut self, doc: &mut RgbaImage) -> bool {
        let Some(change) = self.undo.pop() else {
            return false;
        };
        paste(doc, change.x, change.y, &change.before);
        self.redo.push(change);
        true
    }

    pub fn redo(&mut self, doc: &mut RgbaImage) -> bool {
        let Some(change) = self.redo.pop() else {
            return false;
        };
        paste(doc, change.x, change.y, &change.after);
        self.undo.push(change);
        true
    }
}

fn diff_bbox(a: &RgbaImage, b: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = a.dimensions();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (w, h, 0, 0);
    let mut found = false;
    for y in 0..h {
        for x in 0..w {
            if a.get_pixel(x, y) != b.get_pixel(x, y) {
                found = true;
                if x < min_x {
                    min_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if x > max_x {
                    max_x = x;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }
    if !found {
        return None;
    }
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    Some((min_x, min_y, width, height))
}

fn crop(img: &RgbaImage, x: u32, y: u32, w: u32, h: u32) -> RgbaImage {
    image::imageops::crop_imm(img, x, y, w, h).to_image()
}

fn paste(doc: &mut RgbaImage, x: u32, y: u32, patch: &RgbaImage) {
    for (py, row) in patch.rows().enumerate() {
        for (px, p) in row.enumerate() {
            let (dx, dy) = (x + px as u32, y + py as u32);
            if dx < doc.width() && dy < doc.height() {
                doc.put_pixel(dx, dy, *p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, image::Rgba([255, 255, 255, 255]))
    }

    #[test]
    fn undo_redo_roundtrip() {
        let before = blank(8, 8);
        let mut after = before.clone();
        after.put_pixel(3, 4, image::Rgba([255, 0, 0, 255]));
        let mut history = History::default();
        history.push(&before, &after);

        let mut doc = after.clone();
        assert!(history.undo(&mut doc));
        assert_eq!(doc.get_pixel(3, 4), &image::Rgba([255, 255, 255, 255]));
        assert!(history.redo(&mut doc));
        assert_eq!(doc.get_pixel(3, 4), &image::Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn empty_diff_is_ignored() {
        let a = blank(4, 4);
        let mut history = History::default();
        history.push(&a, &a.clone());
        assert!(!history.can_undo());
    }

    #[test]
    fn stores_only_tight_region() {
        let before = blank(100, 100);
        let mut after = before.clone();
        after.put_pixel(50, 60, image::Rgba([0, 0, 0, 255]));
        let mut history = History::default();
        history.push(&before, &after);
        assert_eq!(history.undo[0].x, 50);
        assert_eq!(history.undo[0].y, 60);
        assert_eq!(history.undo[0].before.dimensions(), (1, 1));
    }

    #[test]
    fn new_push_clears_redo() {
        let before = blank(4, 4);
        let mut after = before.clone();
        after.put_pixel(0, 0, image::Rgba([1, 2, 3, 255]));
        let mut history = History::default();
        history.push(&before, &after);
        let mut doc = after.clone();
        history.undo(&mut doc);
        assert!(history.can_redo());
        history.push(&before, &after);
        assert!(!history.can_redo());
    }
}
