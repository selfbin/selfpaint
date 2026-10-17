//! Tool selection and stroke-preview state.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
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

impl Tool {
    pub fn as_str(self) -> &'static str {
        match self {
            Tool::Pencil => "pencil",
            Tool::Brush => "brush",
            Tool::Eraser => "eraser",
            Tool::Bucket => "bucket",
            Tool::Text => "text",
            Tool::Picker => "picker",
            Tool::Line => "line",
            Tool::Rect => "rect",
            Tool::Ellipse => "ellipse",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pencil" => Tool::Pencil,
            "eraser" => Tool::Eraser,
            "bucket" => Tool::Bucket,
            "text" => Tool::Text,
            "picker" => Tool::Picker,
            "line" => Tool::Line,
            "rect" => Tool::Rect,
            "ellipse" => Tool::Ellipse,
            _ => Tool::Brush,
        }
    }

    /// Tools that paint freehand strokes while dragging.
    pub fn is_stroke(self) -> bool {
        matches!(self, Tool::Pencil | Tool::Brush | Tool::Eraser)
    }

    /// Tools that drag out a geometric shape preview.
    pub fn is_shape(self) -> bool {
        matches!(self, Tool::Line | Tool::Rect | Tool::Ellipse)
    }
}

/// Live shape preview between drag start and release.
#[derive(Clone, Copy, Debug)]
pub struct ShapePreview {
    pub shape: Tool,
    pub start: (f32, f32),
    pub current: (f32, f32),
    pub color: [u8; 4],
    pub filled: bool,
    pub thickness: f32,
}
