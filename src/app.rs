//! Application state, panels, canvas interaction and file jobs.

#[cfg(target_arch = "wasm32")]
use crate::time::wasm_time::InstantExt;
#[cfg(target_arch = "wasm32")]
use crate::time::{Duration, Instant};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
use craft_settings::{Location, SaveStatus, SettingsStore};
use egui::{
    Color32, ColorImage, FontId, Modifiers, PointerButton, Pos2, Rect, RichText, Sense, Stroke,
    TextureOptions, Vec2,
};

use crate::canvas::{self, CanvasView};
use crate::document::Document;
use crate::fonts::{FontCatalog, style_from_flags};
use crate::history::History;
use crate::i18n::{L10n, Lang};
use crate::palette::{
    ACCENT, ACCENT_HOVER, BORDER, CANVAS_BG, CHROME_BG, STATUS_BG, TEXT, TEXT_MUTED,
};
use crate::project::{self, ProjectMeta};
use crate::ribbon::{ActiveColor, RibbonEvent, RibbonState};
use crate::settings::Prefs;
#[cfg(target_arch = "wasm32")]
use crate::settings::{Location, SaveStatus, SettingsStore};
use crate::tools::{ShapePreview, Tool};

const DEFAULT_DOC_W: u32 = 1920;
const DEFAULT_DOC_H: u32 = 1200;
fn status_ttl() -> Duration {
    Duration::from_secs(6)
}

enum DragState {
    None,
    Stroke {
        button: PointerButton,
        before: image::RgbaImage,
        last: (f32, f32),
    },
    Shape {
        button: PointerButton,
        start: (f32, f32),
    },
}

struct TextState {
    draft: String,
    anchor: (f32, f32),
    editing: bool,
    focus_pending: bool,
}

enum FileResult {
    SavedPng(PathBuf),
    SavedProject(PathBuf),
    OpenedImage {
        path: PathBuf,
        pixels: image::RgbaImage,
    },
    OpenedProject {
        path: PathBuf,
        pixels: image::RgbaImage,
        meta: ProjectMeta,
    },
    Failed {
        message: String,
    },
    Cancelled,
}

pub struct SelfPaintApp {
    doc: Document,
    history: History,
    view: CanvasView,
    tool: Tool,
    stroke: f32,
    color1: Color32,
    color2: Color32,
    active_color: ActiveColor,
    filled_shapes: bool,
    lang: Lang,
    l10n: L10n,
    settings: SettingsStore<Prefs>,
    texture: Option<egui::TextureHandle>,
    texture_dirty: bool,
    drag: DragState,
    shape_preview: Option<ShapePreview>,
    text: TextState,
    file_rx: Option<Receiver<FileResult>>,
    current_png: Option<PathBuf>,
    current_project: Option<PathBuf>,
    status: Option<(String, Instant)>,
    show_new_dialog: bool,
    new_w: u32,
    new_h: u32,
    show_settings: bool,
    show_splash: bool,
    fonts: FontCatalog,
    text_size: f32,
    text_font: usize,
    text_bold: bool,
    text_italic: bool,
preview_font_key: Option<(String, crate::fonts::FontStyle)>,

    /// egui context for accessing viewport information
    egui_ctx: egui::Context,
}

impl SelfPaintApp {
    pub fn apply_visuals(&self, ctx: &egui::Context) {
        let theme = crate::palette::Theme::from_code(&self.settings.get().general.theme);
        crate::palette::set_current(theme);
        let mut visuals = egui::Visuals::light();
        crate::palette::apply_theme(&mut visuals, theme);
        ctx.set_visuals(visuals);
    }

    pub fn new() -> Self {
        let (settings, _report) = SettingsStore::<Prefs>::open(Location::Auto);
        let prefs = settings.get().clone();
        let lang = Lang::from_code(&prefs.general.language);
        let mut view = CanvasView::default();
        view.set_zoom(prefs.view.zoom);
        let fonts = FontCatalog::new();
        let text_font = fonts.family_index(&prefs.tools.text_font);
        Self {
            doc: Document::new(DEFAULT_DOC_W, DEFAULT_DOC_H),
            history: History::default(),
            view,
            tool: Tool::from_str(&prefs.tools.tool),
            stroke: prefs.tools.stroke.clamp(1, 64) as f32,
            color1: rgba_to_color32(prefs.tools.color1),
            color2: rgba_to_color32(prefs.tools.color2),
            active_color: active_from_index(prefs.tools.active_color),
            filled_shapes: prefs.tools.filled_shapes,
            lang,
            l10n: L10n::of(lang),
            settings,
            texture: None,
            texture_dirty: true,
            drag: DragState::None,
            shape_preview: None,
            text: TextState {
                draft: String::new(),
                anchor: (0.0, 0.0),
                editing: false,
                focus_pending: false,
            },
            file_rx: None,
            current_png: None,
            current_project: None,
            status: None,
            show_new_dialog: false,
            new_w: DEFAULT_DOC_W,
            new_h: DEFAULT_DOC_H,
            show_settings: false,
            show_splash: true,
            fonts,
            text_size: prefs.tools.text_size.clamp(8, 256) as f32,
            text_font,
            text_bold: prefs.tools.text_bold,
            text_italic: prefs.tools.text_italic,
            preview_font_key: None,

            egui_ctx: egui::Context::default(),
        }
    }

    fn status(&mut self, message: String) {
        self.status = Some((message, Instant::now()));
    }

    fn persist(&mut self, f: impl FnOnce(&mut Prefs)) {
        self.settings.update(f);
    }

    fn mark_doc_dirty(&mut self) {
        self.texture_dirty = true;
    }

    fn replace_document(&mut self, doc: Document) {
        self.doc = doc;
        self.history.clear();
        self.texture = None;
        self.mark_doc_dirty();
        self.text.editing = false;
    }

    fn undo(&mut self) {
        if self.history.undo(&mut self.doc.pixels) {
            self.mark_doc_dirty();
        }
    }

    fn redo(&mut self) {
        if self.history.redo(&mut self.doc.pixels) {
            self.mark_doc_dirty();
        }
    }

    // ---------------------------------------------------------------- files

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_save_png(&mut self) {
        if self.file_rx.is_some() {
            return;
        }
        let (tx, rx) = channel();
        self.file_rx = Some(rx);
        let pixels = self.doc.pixels.clone();
        let dir = self
            .current_png
            .clone()
            .or_else(|| non_empty_dir(&self.settings.get().tools.last_dir));
        let name = self
            .current_png
            .as_ref()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
            .unwrap_or_else(|| format!("{}.png", self.l10n.untitled));
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("PNG", &["png"])
                .set_file_name(name);
            if let Some(dir) = dir {
                dialog = dialog.set_directory(dir);
            }
            let Some(path) = dialog.save_file() else {
                let _ = tx.send(FileResult::Cancelled);
                return;
            };
            let msg = match pixels.save_with_format(&path, image::ImageFormat::Png) {
                Ok(()) => FileResult::SavedPng(path),
                Err(e) => FileResult::Failed {
                    message: e.to_string(),
                },
            };
            let _ = tx.send(msg);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_save_png(&mut self) {
        self.status("Saving unavailable in web build".to_string());
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_save_project(&mut self) {
        if self.file_rx.is_some() {
            return;
        }
        let (tx, rx) = channel();
        self.file_rx = Some(rx);
        let meta = self.project_meta();
        let pixels = self.doc.pixels.clone();
        let dir = self
            .current_project
            .clone()
            .or_else(|| non_empty_dir(&self.settings.get().tools.last_dir));
        let name = self
            .current_project
            .as_ref()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
            .unwrap_or_else(|| format!("{}.sproj", self.l10n.untitled));
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("SelfPaint project", &["sproj"])
                .set_file_name(name);
            if let Some(dir) = dir {
                dialog = dialog.set_directory(dir);
            }
            let Some(path) = dialog.save_file() else {
                let _ = tx.send(FileResult::Cancelled);
                return;
            };
            let msg = match project::encode(&pixels, &meta) {
                Ok(bytes) => match std::fs::write(&path, bytes) {
                    Ok(()) => FileResult::SavedProject(path),
                    Err(e) => FileResult::Failed {
                        message: e.to_string(),
                    },
                },
                Err(e) => FileResult::Failed {
                    message: e.to_string(),
                },
            };
            let _ = tx.send(msg);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_save_project(&mut self) {
        self.status("Saving unavailable in web build".to_string());
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_open(&mut self) {
        if self.file_rx.is_some() {
            return;
        }
        let (tx, rx) = channel();
        self.file_rx = Some(rx);
        let dir = non_empty_dir(&self.settings.get().tools.last_dir);
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("Images & projects", &["png", "jpg", "jpeg", "sproj"])
                .add_filter("PNG", &["png"])
                .add_filter("JPEG", &["jpg", "jpeg"])
                .add_filter("SelfPaint project", &["sproj"]);
            if let Some(dir) = dir {
                dialog = dialog.set_directory(dir);
            }
            let Some(path) = dialog.pick_file() else {
                let _ = tx.send(FileResult::Cancelled);
                return;
            };
            let msg = match std::fs::read(&path) {
                Ok(bytes) if project::is_project(&bytes) => match project::decode(&bytes) {
                    Ok((pixels, meta)) => FileResult::OpenedProject { path, pixels, meta },
                    Err(e) => FileResult::Failed {
                        message: e.to_string(),
                    },
                },
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(img) => FileResult::OpenedImage {
                        path,
                        pixels: img.into_rgba8(),
                    },
                    Err(e) => FileResult::Failed {
                        message: e.to_string(),
                    },
                },
                Err(e) => FileResult::Failed {
                    message: e.to_string(),
                },
            };
            let _ = tx.send(msg);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_open(&mut self) {
        self.status("File dialogs unavailable in web build".to_string());
    }

    fn project_meta(&self) -> ProjectMeta {
        ProjectMeta {
            lang: self.lang.code().into(),
            tool: self.tool.as_str().into(),
            stroke: self.stroke.clamp(1.0, 64.0) as u8,
            color1: color32_to_rgba(self.color1),
            color2: color32_to_rgba(self.color2),
            active_color: match self.active_color {
                ActiveColor::One => 1,
                ActiveColor::Two => 2,
            },
            filled_shapes: self.filled_shapes,
            zoom: self.view.zoom,
            text_size: self.text_size.clamp(8.0, 256.0) as u16,
            text_font: self.text_font_name().to_owned(),
            text_bold: self.text_bold,
            text_italic: self.text_italic,
        }
    }

    fn text_font_name(&self) -> &str {
        self.fonts
            .family_name(self.text_font)
            .unwrap_or("Open Sans")
    }

    fn remember_dir(&mut self, path: &std::path::Path) {
        let dir = path
            .parent()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.persist(|p| p.tools.last_dir = dir);
    }

    fn poll_file_results(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.file_rx.take() else {
            return;
        };
        let mut _done = false;
        while let Ok(result) = rx.try_recv() {
            match result {
                FileResult::SavedPng(path) => {
                    self.remember_dir(&path);
                    self.current_png = Some(path.clone());
                    self.status(format!("{} {}", self.l10n.saved_ok, path.display()));
                }
                FileResult::SavedProject(path) => {
                    self.remember_dir(&path);
                    self.current_project = Some(path.clone());
                    self.status(format!("{} {}", self.l10n.project_saved_ok, path.display()));
                }
                FileResult::OpenedImage { path, pixels } => {
                    self.remember_dir(&path);
                    self.current_png = Some(path.clone());
                    self.current_project = None;
                    self.replace_document(Document::from_rgba(pixels));
                    self.status(format!("{} {}", self.l10n.opened_ok, path.display()));
                }
                FileResult::OpenedProject { path, pixels, meta } => {
                    self.remember_dir(&path);
                    self.current_project = Some(path.clone());
                    self.current_png = None;
                    self.replace_document(Document::from_rgba(pixels));
                    self.apply_meta(meta);
                    self.status(format!(
                        "{} {}",
                        self.l10n.project_opened_ok,
                        path.display()
                    ));
                }
                FileResult::Failed { message } => {
                    self.status(self.l10n.err_save.replace("{}", &message));
                }
                FileResult::Cancelled => {
                    _done = true;
                }
            }
            _done = true;
        }
        if _done {
            self.file_rx = None;
        } else {
            self.file_rx = Some(rx);
        }
        if self.file_rx.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn apply_meta(&mut self, meta: ProjectMeta) {
        self.lang = Lang::from_code(&meta.lang);
        self.l10n = L10n::of(self.lang);
        self.tool = Tool::from_str(&meta.tool);
        self.stroke = (meta.stroke as f32).clamp(1.0, 64.0);
        self.color1 = rgba_to_color32(meta.color1);
        self.color2 = rgba_to_color32(meta.color2);
        self.active_color = active_from_index(meta.active_color);
        self.filled_shapes = meta.filled_shapes;
        self.view.set_zoom(meta.zoom);
        self.text_size = (meta.text_size as f32).clamp(8.0, 256.0);
        self.text_font = self.fonts.family_index(&meta.text_font);
        self.text_bold = meta.text_bold;
        self.text_italic = meta.text_italic;
        self.preview_font_key = None;
        let lang = self.lang.code();
        let tool = self.tool.as_str();
        let stroke = self.stroke;
        let color1 = color32_to_rgba(self.color1);
        let color2 = color32_to_rgba(self.color2);
        let active = match self.active_color {
            ActiveColor::One => 1,
            ActiveColor::Two => 2,
        };
        let filled = self.filled_shapes;
        let zoom = self.view.zoom;
        let text_size = self.text_size.clamp(8.0, 256.0) as u16;
        let text_font = self.text_font_name().to_owned();
        let text_bold = self.text_bold;
        let text_italic = self.text_italic;
        self.persist(move |p| {
            p.general.language = lang.into();
            p.tools.tool = tool.into();
            p.tools.stroke = stroke as u8;
            p.tools.color1 = color1;
            p.tools.color2 = color2;
            p.tools.active_color = active;
            p.tools.filled_shapes = filled;
            p.view.zoom = zoom;
            p.tools.text_size = text_size;
            p.tools.text_font = text_font;
            p.tools.text_bold = text_bold;
            p.tools.text_italic = text_italic;
        });
    }

    // ------------------------------------------------------------ shortcuts

    fn handle_shortcuts(&mut self, ui: &mut egui::Ui) {
        let shift_z =
            ui.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, egui::Key::Z));
        let z = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, egui::Key::Z));
        let y = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, egui::Key::Y));
        let s = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, egui::Key::S));
        let o = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, egui::Key::O));
        let n = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, egui::Key::N));

        if z {
            self.undo();
        }
        if y || shift_z {
            self.redo();
        }
        if s {
            self.spawn_save_png();
        }
        if o {
            self.spawn_open();
        }
        if n {
            self.show_new_dialog = true;
        }
    }

    // --------------------------------------------------------------- canvas

    fn ensure_texture(&mut self, ctx: &egui::Context) {
        let size = [self.doc.width() as usize, self.doc.height() as usize];
        if let Some(handle) = &mut self.texture
            && handle.size() == size
        {
            if self.texture_dirty {
                let image = ColorImage::from_rgba_unmultiplied(size, self.doc.pixels.as_raw());
                handle.set(image, TextureOptions::LINEAR);
                self.texture_dirty = false;
            }
            return;
        }
        let image = ColorImage::from_rgba_unmultiplied(size, self.doc.pixels.as_raw());
        self.texture = Some(ctx.load_texture("selfpaint-doc", image, TextureOptions::LINEAR));
        self.texture_dirty = false;
    }

    /// Register the selected family/style into egui under `FontFamily::Name`
    /// so the live TextEdit preview matches what gets rasterized onto the canvas.
    fn sync_preview_font(&mut self, ctx: &egui::Context) {
        let Some(name) = self.fonts.family_name(self.text_font).map(str::to_owned) else {
            return;
        };
        let style = style_from_flags(self.text_bold, self.text_italic);
        if self.preview_font_key.as_ref() == Some(&(name.clone(), style)) {
            return;
        }
        let Some(bytes) = self.fonts.bytes(&name, style) else {
            return;
        };
        let mut defs = ctx.fonts(|f| f.definitions().clone());
        defs.font_data.remove("selfpaint-preview");
        defs.font_data.insert(
            "selfpaint-preview".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes.as_ref().to_vec())),
        );
        defs.families.insert(
            egui::FontFamily::Name("SelfPaint/Text".into()),
            vec!["selfpaint-preview".to_owned()],
        );
        ctx.set_fonts(defs);
        self.preview_font_key = Some((name, style));
    }

    fn brush_radius(&self) -> f32 {
        match self.tool {
            Tool::Pencil => 0.5,
            _ => (self.stroke * 0.5).max(0.5),
        }
    }

    /// Color used by freehand tools for a given mouse button.
    fn stroke_color(&self, button: PointerButton) -> [u8; 4] {
        if self.tool == Tool::Eraser {
            return [255, 255, 255, 255];
        }
        match button {
            PointerButton::Secondary => color32_to_rgba(self.color2),
            _ => color32_to_rgba(self.color1),
        }
    }

    fn show_canvas(&mut self, ui: &mut egui::Ui) {
        self.ensure_texture(ui.ctx());
        if self.tool == Tool::Text {
            self.sync_preview_font(ui.ctx());
        }

        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        let sheet = self
            .view
            .sheet_rect(rect, self.doc.width() as f32, self.doc.height() as f32);
        canvas::paint_sheet(&painter, rect, sheet);

        if let Some(handle) = &self.texture {
            let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            painter.image(handle.id(), sheet, uv, Color32::WHITE);
        }

        // Middle-drag pan.
        if response.dragged_by(PointerButton::Middle) {
            self.view.pan += response.drag_delta();
            self.text.editing = false;
        }

        // Ctrl+scroll zoom around the pointer.
        if response.hovered() {
            let (scroll, ctrl) = ui.input(|i| (i.smooth_scroll_delta.y, i.modifiers.ctrl));
            if ctrl
                && scroll.abs() > 0.0
                && let Some(pointer) = ui.input(|i| i.pointer.hover_pos())
            {
                let factor = (1.0 + scroll * 0.002).clamp(0.5, 2.0);
                self.zoom_at(sheet, pointer, self.view.zoom * factor);
                self.text.editing = false;
            }
        }

        let pointer_canvas = response
            .interact_pointer_pos()
            .or_else(|| ui.input(|i| i.pointer.hover_pos()))
            .and_then(|p| self.view.to_canvas(sheet, p));

        self.handle_tool_input(ui, &response, sheet, pointer_canvas);

        if let Some(preview) = self.shape_preview {
            canvas::paint_shape_preview(&painter, sheet, &self.view, &preview);
        }

        if self.tool.is_stroke()
            && !matches!(self.drag, DragState::Stroke { .. })
            && response.hovered()
            && !self.text.editing
            && let Some(hover) = ui.input(|i| i.pointer.hover_pos())
        {
            let r = self.brush_radius() * self.view.zoom;
            canvas::paint_brush_cursor(&painter, hover, r, Color32::from_black_alpha(200));
        }

        self.show_text_editor(ui, sheet);
    }

    fn zoom_at(&mut self, sheet: Rect, pointer: Pos2, new_zoom: f32) {
        let Some(old) = self.view.to_canvas(sheet, pointer) else {
            return;
        };
        self.view.set_zoom(new_zoom);
        let after = sheet.min + Vec2::new(old.0 * self.view.zoom, old.1 * self.view.zoom);
        self.view.pan += pointer - after;
    }

    fn handle_tool_input(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        sheet: Rect,
        pointer: Option<(f32, f32)>,
    ) {
        if self.text.editing {
            return;
        }

        let secondary_pressed = ui.input(|i| i.pointer.button_pressed(PointerButton::Secondary));
        let secondary_released = ui.input(|i| i.pointer.button_released(PointerButton::Secondary));

        let draggable = self.tool.is_stroke() || self.tool.is_shape();

        match self.drag {
            DragState::None => {
                if response.drag_started() && draggable {
                    if let Some(pos) = pointer {
                        self.begin_stroke_or_shape(PointerButton::Primary, pos);
                    }
                } else if secondary_pressed && response.hovered() && draggable {
                    if let Some(pos) = pointer {
                        self.begin_stroke_or_shape(PointerButton::Secondary, pos);
                    }
                } else if response.clicked() {
                    if let Some(pos) = pointer {
                        self.click_tool(pos, PointerButton::Primary);
                    }
                } else if response.secondary_clicked() {
                    let pos = pointer.or_else(|| {
                        ui.input(|i| i.pointer.hover_pos())
                            .and_then(|p| self.view.to_canvas(sheet, p))
                    });
                    if let Some(pos) = pos {
                        self.click_tool(pos, PointerButton::Secondary);
                    }
                }
            }
            DragState::Stroke { .. } | DragState::Shape { .. } => {}
        }

        let shift = ui.input(|i| i.modifiers.shift);
        match std::mem::replace(&mut self.drag, DragState::None) {
            DragState::None => {}
            DragState::Stroke {
                button,
                before,
                last,
            } => {
                let released = match button {
                    PointerButton::Secondary => secondary_released,
                    _ => response.drag_stopped(),
                };
                if !released {
                    if let Some(pos) = pointer {
                        if last != pos {
                            self.apply_stroke_segment(last, pos, button);
                        }
                        self.drag = DragState::Stroke {
                            button,
                            before,
                            last: pos,
                        };
                    } else {
                        self.drag = DragState::Stroke {
                            button,
                            before,
                            last,
                        };
                    }
                } else {
                    self.history.push(&before, &self.doc.pixels);
                    self.mark_doc_dirty();
                }
            }
            DragState::Shape { button, start } => {
                let released = match button {
                    PointerButton::Secondary => secondary_released,
                    _ => response.drag_stopped(),
                };
                if !released {
                    if let Some(pos) = pointer {
                        let shape = self.shape_preview.map(|p| p.shape).unwrap_or(self.tool);
                        let constrained = canvas::constrain_shift(start, pos, shift, shape);
                        if let Some(preview) = &mut self.shape_preview {
                            preview.current = constrained;
                        }
                    }
                    self.drag = DragState::Shape { button, start };
                } else if let Some(preview) = self.shape_preview.take() {
                    let before = self.doc.pixels.clone();
                    self.apply_shape(preview);
                    self.history.push(&before, &self.doc.pixels);
                    self.mark_doc_dirty();
                }
            }
        }
    }

    fn begin_stroke_or_shape(&mut self, button: PointerButton, pos: (f32, f32)) {
        if self.tool.is_stroke() {
            let before = self.doc.pixels.clone();
            let color = self.stroke_color(button);
            let radius = self.brush_radius();
            self.doc.stamp_disk(pos.0, pos.1, radius, color);
            self.drag = DragState::Stroke {
                button,
                before,
                last: pos,
            };
        } else if self.tool.is_shape() {
            let color = match button {
                PointerButton::Secondary => color32_to_rgba(self.color2),
                _ => color32_to_rgba(self.color1),
            };
            self.shape_preview = Some(ShapePreview {
                shape: self.tool,
                start: pos,
                current: pos,
                color,
                filled: self.filled_shapes,
                thickness: self.stroke,
            });
            self.drag = DragState::Shape { button, start: pos };
        }
    }

    fn apply_stroke_segment(&mut self, from: (f32, f32), to: (f32, f32), button: PointerButton) {
        let color = self.stroke_color(button);
        let radius = self.brush_radius();
        self.doc.stroke_segment(from, to, radius, color);
    }

    fn apply_shape(&mut self, preview: ShapePreview) {
        let r = (preview.thickness * 0.5).max(0.5);
        match preview.shape {
            Tool::Line => self
                .doc
                .stroke_segment(preview.start, preview.current, r, preview.color),
            Tool::Rect => self.doc.draw_rect(
                preview.start,
                preview.current,
                r,
                preview.color,
                preview.filled,
            ),
            Tool::Ellipse => self.doc.draw_ellipse(
                preview.start,
                preview.current,
                r,
                preview.color,
                preview.filled,
            ),
            _ => {}
        }
    }

    fn click_tool(&mut self, pos: (f32, f32), button: PointerButton) {
        match self.tool {
            Tool::Bucket => {
                let color = match button {
                    PointerButton::Secondary => color32_to_rgba(self.color2),
                    _ => color32_to_rgba(self.color1),
                };
                let before = self.doc.pixels.clone();
                if self
                    .doc
                    .flood_fill(pos.0.floor() as u32, pos.1.floor() as u32, color)
                {
                    self.history.push(&before, &self.doc.pixels);
                    self.mark_doc_dirty();
                }
            }
            Tool::Picker => {
                let x = pos.0.floor() as u32;
                let y = pos.1.floor() as u32;
                if x < self.doc.width() && y < self.doc.height() {
                    let px = self.doc.pixels.get_pixel(x, y).0;
                    let color = Color32::from_rgba_unmultiplied(px[0], px[1], px[2], px[3]);
                    match button {
                        PointerButton::Secondary => self.color2 = color,
                        _ => self.color1 = color,
                    }
                }
            }
            Tool::Text => {
                self.text.editing = true;
                self.text.draft.clear();
                self.text.anchor = pos;
                self.text.focus_pending = true;
            }
            _ => {}
        }
    }

    fn show_text_editor(&mut self, ui: &mut egui::Ui, sheet: Rect) {
        if !self.text.editing {
            return;
        }
        let anchor_screen = self.view.to_screen(sheet, self.text.anchor);
        let mut apply = false;
        let mut cancel = false;
        let focus_pending = self.text.focus_pending;
        egui::Area::new(ui.id().with("selfpaint-text-editor"))
            .fixed_pos(anchor_screen + Vec2::new(0.0, 4.0))
            .show(ui, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(CHROME_BG())
                    .stroke(Stroke::new(1.0, ACCENT()))
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::new(320.0, 90.0));
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.text.draft)
                                .font(FontId::new(
                                    self.text_size.clamp(8.0, 256.0),
                                    egui::FontFamily::Name("SelfPaint/Text".into()),
                                ))
                                .desired_width(300.0)
                                .desired_rows(3)
                                .hint_text(self.l10n.text_hint),
                        );
                        if focus_pending {
                            response.request_focus();
                        }
                        ui.horizontal(|ui| {
                            if ui.button(self.l10n.apply).clicked() {
                                apply = true;
                            }
                            if ui.button(self.l10n.cancel).clicked() {
                                cancel = true;
                            }
                        });
                    });
            });
        self.text.focus_pending = false;
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            cancel = true;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command) {
            apply = true;
        }
        if apply {
            self.commit_text();
            self.text.editing = false;
        } else if cancel {
            self.text.editing = false;
        }
    }

    fn commit_text(&mut self) {
        let text = self.text.draft.clone();
        let text = text.trim_end();
        if text.is_empty() {
            return;
        }
        let size = self.text_size.clamp(8.0, 256.0);
        let color = color32_to_rgba(self.color1);
        let family = self.text_font_name().to_owned();
        let style = style_from_flags(self.text_bold, self.text_italic);
        let Some(font) = self.fonts.load(&family, style) else {
            return;
        };
        let Some((buffer, w, h)) = rasterize_text(&font, text, size, color) else {
            return;
        };
        let before = self.doc.pixels.clone();
        self.doc.blit_rgba(
            self.text.anchor.0.floor() as i64,
            self.text.anchor.1.floor() as i64,
            &buffer,
            w as usize,
            h as usize,
        );
        self.history.push(&before, &self.doc.pixels);
        self.mark_doc_dirty();
    }

    // ------------------------------------------------------------ statusbar

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        let (w, h) = (self.doc.width(), self.doc.height());
        let status_msg = self
            .status
            .as_ref()
            .filter(|(_, t)| {
                #[cfg(target_arch = "wasm32")]
                {
                    t.elapsed_wasm() < status_ttl()
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    t.elapsed() < status_ttl()
                }
            })
            .map(|(m, _)| m.clone());

        egui::Panel::bottom("selfpaint-status")
            .exact_size(28.0)
            .frame(
                egui::Frame::NONE
                    .fill(STATUS_BG())
                    .inner_margin(egui::Margin::symmetric(10, 4)),
            )
            .show(ui, |ui| {
                let mut fit_clicked = false;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{w} × {h} px"))
                            .size(12.0)
                            .color(TEXT()),
                    );
                    if let Some(msg) = status_msg {
                        ui.separator();
                        ui.label(RichText::new(msg).size(12.0).color(TEXT_MUTED()));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        fit_clicked = ui.small_button(self.l10n.fit).clicked();
                        ui.spacing_mut().slider_width = 140.0;
                        let mut pct = (self.view.zoom * 100.0).round() as i32;
                        let slider = ui.add(
                            egui::Slider::new(&mut pct, 25..=800)
                                .fixed_decimals(0)
                                .suffix("%"),
                        );
                        if slider.changed() {
                            self.view.set_zoom(pct as f32 / 100.0);
                        }
                        if ui.small_button("+").clicked() {
                            self.view.set_zoom(self.view.zoom * 1.25);
                        }
                        if ui.small_button("−").clicked() {
                            self.view.set_zoom(self.view.zoom / 1.25);
                        }
                        ui.label(RichText::new(self.l10n.zoom).size(12.0).color(TEXT_MUTED()));
                    });
                });
                if fit_clicked {
                    let avail = ui.available_size_before_wrap();
                    let fit = self.view.fit_zoom(
                        Rect::from_min_size(Pos2::ZERO, avail),
                        self.doc.width() as f32,
                        self.doc.height() as f32,
                    );
                    self.view.set_zoom(fit);
                    self.view.pan = Vec2::ZERO;
                }
            });
    }

    // ----------------------------------------------------------- new dialog

    fn show_new_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_new_dialog;
        let mut create = false;
        let mut cancelled = false;
        egui::Window::new(self.l10n.new_title)
            .id(egui::Id::new("selfpaint-new"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(self.l10n.width);
                    ui.add(egui::DragValue::new(&mut self.new_w).range(1..=8192));
                    ui.label(self.l10n.height);
                    ui.add(egui::DragValue::new(&mut self.new_h).range(1..=8192));
                });
                ui.horizontal(|ui| {
                    for (label, w, h) in [
                        ("1920 × 1200", 1920, 1200),
                        ("1920 × 1080", 1920, 1080),
                        ("1280 × 720", 1280, 720),
                        ("800 × 600", 800, 600),
                    ] {
                        if ui.small_button(label).clicked() {
                            self.new_w = w;
                            self.new_h = h;
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(self.l10n.ok).clicked() {
                        create = true;
                    }
                    if ui.button(self.l10n.cancel).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled {
            open = false;
        }
        self.show_new_dialog = open;
        if create {
            let doc = Document::new(self.new_w, self.new_h);
            self.replace_document(doc);
            self.current_png = None;
            self.current_project = None;
            self.view.pan = Vec2::ZERO;
            self.status(format!("{} × {} px", self.new_w, self.new_h));
        }
    }

    // -------------------------------------------------------- settings dialog

    fn show_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings;
        egui::Window::new(self.l10n.settings_title)
            .id(egui::Id::new("selfpaint-settings"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(self.l10n.language)
                        .size(13.0)
                        .color(TEXT_MUTED()),
                );
                ui.horizontal(|ui| {
                    let pt = ui.selectable_label(self.lang == Lang::Pt, "Português");
                    let en = ui.selectable_label(self.lang == Lang::En, "English");
                    if pt.clicked() && self.lang != Lang::Pt {
                        self.lang = Lang::Pt;
                        self.l10n = L10n::of(Lang::Pt);
                        self.persist(|p| p.general.language = "pt".into());
                    }
                    if en.clicked() && self.lang != Lang::En {
                        self.lang = Lang::En;
                        self.l10n = L10n::of(Lang::En);
                        self.persist(|p| p.general.language = "en".into());
                    }
                });
                ui.separator();
                ui.label(RichText::new(self.l10n.theme).size(13.0).color(TEXT_MUTED()));
                ui.horizontal(|ui| {
                    let light = ui.selectable_label(
                        self.settings.get().general.theme == "light",
                        self.l10n.theme_light,
                    );
                    let dark = ui.selectable_label(
                        self.settings.get().general.theme == "dark",
                        self.l10n.theme_dark,
                    );
                    let gray = ui.selectable_label(
                        self.settings.get().general.theme == "gray",
                        self.l10n.theme_gray,
                    );
                    let black = ui.selectable_label(
                        self.settings.get().general.theme == "black",
                        self.l10n.theme_black,
                    );
                    if light.clicked() && self.settings.get().general.theme != "light" {
                        self.persist(|p| p.general.theme = "light".into());
                        self.apply_visuals(ui.ctx());
                    }
                    if dark.clicked() && self.settings.get().general.theme != "dark" {
                        self.persist(|p| p.general.theme = "dark".into());
                        self.apply_visuals(ui.ctx());
                    }
                    if gray.clicked() && self.settings.get().general.theme != "gray" {
                        self.persist(|p| p.general.theme = "gray".into());
                        self.apply_visuals(ui.ctx());
                    }
                    if black.clicked() && self.settings.get().general.theme != "black" {
                        self.persist(|p| p.general.theme = "black".into());
                        self.apply_visuals(ui.ctx());
                    }
                    // Breeze themes
                    let breeze_light = ui.selectable_label(
                        self.settings.get().general.theme == "breeze-light",
                        self.l10n.theme_breeze_light,
                    );
                    let breeze_dark = ui.selectable_label(
                        self.settings.get().general.theme == "breeze-dark",
                        self.l10n.theme_breeze_dark,
                    );
                    if breeze_light.clicked() && self.settings.get().general.theme != "breeze-light" {
                        self.persist(|p| p.general.theme = "breeze-light".into());
                        self.apply_visuals(ui.ctx());
                    }
                    if breeze_dark.clicked() && self.settings.get().general.theme != "breeze-dark" {
                        self.persist(|p| p.general.theme = "breeze-dark".into());
                        self.apply_visuals(ui.ctx());
                    }
                });
            });
        self.show_settings = open;
    }

    // ------------------------------------------------------------ splash screen

    fn show_splash_screen(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let (splash_rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
        let painter = ui.painter_at(splash_rect);

        // Dim the app behind the splash.\n        // Use light theme background color\n        painter.rect_filled(splash_rect, 0.0, Color32::from_rgb(0xEE, 0xEE, 0xEE));

        let center = splash_rect.center();
        let name = self.l10n.splash_name;
        let desc = self.l10n.splash_desc;
        let hint = self.l10n.start_subtitle;
        let start = self.l10n.new_file;

        // Calculate button vertical spacing based on available height to avoid going off-screen
        let available_height = splash_rect.height();
        let button_height = 36.0_f32;
        let button_count = 3.0_f32; // start, open, settings
        let total_buttons_height = button_count * button_height + (button_count - 1.0_f32) * 10.0_f32;
        let max_top_offset = available_height / 2.0_f32 - total_buttons_height / 2.0_f32;
        let start_offset = 40.0_f32.min(max_top_offset.max(20.0_f32));
        let open_offset = start_offset + button_height + 10.0;
        let settings_offset = open_offset + button_height + 10.0;

        // App name.
        painter.text(
            center - Vec2::new(0.0, 60.0),
            egui::Align2::CENTER_CENTER,
            name,
            FontId::proportional(52.0),
            TEXT(),
        );
        // Description.
        painter.text(
            center - Vec2::new(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            desc,
            FontId::proportional(16.0),
            TEXT_MUTED(),
        );

        // "Start painting" button.
        let btn_size = Vec2::new(200.0, 36.0);
        let btn_rect = Rect::from_center_size(center + Vec2::new(0.0, start_offset), btn_size);
        let btn_id = ui.id().with("selfpaint-splash-start");
        let btn_resp = ui.interact(btn_rect, btn_id, Sense::click());
        let btn_fill = if btn_resp.hovered() {
            ACCENT_HOVER()
        } else {
            ACCENT()
        };
        painter.rect_filled(btn_rect, 6.0, btn_fill);
        painter.text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            start,
            FontId::proportional(15.0),
            Color32::WHITE,
        );

        // Open and Settings buttons.
        let open_rect = Rect::from_center_size(center + Vec2::new(0.0, open_offset), Vec2::new(200.0, 30.0));
        let settings_rect = Rect::from_center_size(center + Vec2::new(0.0, settings_offset), Vec2::new(200.0, 30.0));
        let open_resp = ui.interact(open_rect, ui.id().with("splash-open"), Sense::click());
        let settings_resp = ui.interact(settings_rect, ui.id().with("splash-settings"), Sense::click());
        for (r, label, hovered) in [
            (open_rect, self.l10n.open, open_resp.hovered()),
            (settings_rect, self.l10n.settings_title, settings_resp.hovered()),
        ] {
            painter.rect_filled(r, 6.0, if hovered { ACCENT_HOVER() } else { CHROME_BG() });
            painter.text(r.center(), egui::Align2::CENTER_CENTER, label, FontId::proportional(13.0), TEXT());
        }
        if open_resp.clicked() {
            self.spawn_open();
            self.show_splash = false;
        }
        if settings_resp.clicked() {
            self.show_settings = true;
        }

        // Hint at the bottom.
        painter.text(
            center + Vec2::new(0.0, 110.0),
            egui::Align2::CENTER_CENTER,
            hint,
            FontId::proportional(12.0),
            Color32::from_white_alpha(140),
        );

        // Any click (on the background or the button) dismisses the splash.
        if btn_resp.clicked()
            || (response.clicked()
                && !btn_rect.contains(
                    response
                        .interact_pointer_pos()
                        .unwrap_or(Pos2::new(-1.0, -1.0)),
                ))
        {
            self.show_splash = false;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)) {
            self.show_splash = false;
        }
    }
}

impl Default for SelfPaintApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for SelfPaintApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.show_splash {
            self.show_splash_screen(ui);
            self.show_settings_dialog(ui.ctx());
            return;
        }
        // Custom titlebar
        crate::titlebar::show(ui, &self.l10n.untitled);
        crate::titlebar::resize_borders(ui.ctx());

        self.poll_file_results(ui.ctx());
        self.handle_shortcuts(ui);

        #[cfg(not(target_arch = "wasm32"))]
        let tick = self.settings.tick(Instant::now());
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let SaveStatus::Failed(_err) = &tick.status {
                // The store keeps retrying; nothing to show here.
            }
            if let Some(deadline) = tick.next_deadline {
                ui.ctx()
                    .request_repaint_after(deadline.saturating_duration_since(Instant::now()));
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _tick = self.settings.tick(Instant::now());

        let l10n = self.l10n.clone();
        let mut events = Vec::new();
        {
            let mut ribbon_state = RibbonState {
                tool: &mut self.tool,
                stroke: &mut self.stroke,
                color1: &mut self.color1,
                color2: &mut self.color2,
                active_color: &mut self.active_color,
                filled: &mut self.filled_shapes,
                can_undo: self.history.can_undo(),
                can_redo: self.history.can_redo(),
                text_size: &mut self.text_size,
                text_font: &mut self.text_font,
                font_names: self.fonts.families(),
                text_bold: &mut self.text_bold,
                text_italic: &mut self.text_italic,
            };
            let ribbon = egui::Panel::top("selfpaint-ribbon")
                .frame(
                    egui::Frame::NONE
                        .fill(CHROME_BG())
                        .inner_margin(egui::Margin::symmetric(4, 6)),
                )
                .show(ui, |ui| {
                    events = crate::ribbon::show(ui, &l10n, &mut ribbon_state);
                });
            ui.painter().hline(
                ribbon.response.rect.x_range(),
                ribbon.response.rect.bottom(),
                Stroke::new(1.0, BORDER()),
            );
        }

        {
            let tool = self.tool.as_str();
            let stroke = self.stroke.clamp(1.0, 64.0) as u8;
            let color1 = color32_to_rgba(self.color1);
            let color2 = color32_to_rgba(self.color2);
            let active = match self.active_color {
                ActiveColor::One => 1,
                ActiveColor::Two => 2,
            };
            let filled = self.filled_shapes;
            let lang = self.lang.code();
            let zoom = self.view.zoom;
            let text_size = self.text_size.clamp(8.0, 256.0) as u16;
            let text_font = self.text_font_name().to_owned();
            let text_bold = self.text_bold;
            let text_italic = self.text_italic;
            self.persist(move |p| {
                p.tools.tool = tool.into();
                p.tools.stroke = stroke;
                p.tools.color1 = color1;
                p.tools.color2 = color2;
                p.tools.active_color = active;
                p.tools.filled_shapes = filled;
                p.general.language = lang.into();
                p.view.zoom = zoom;
                p.tools.text_size = text_size;
                p.tools.text_font = text_font;
                p.tools.text_bold = text_bold;
                p.tools.text_italic = text_italic;
            });
        }

        for event in events {
            match event {
                RibbonEvent::New => self.show_new_dialog = true,
                RibbonEvent::Open => self.spawn_open(),
                RibbonEvent::SavePng => self.spawn_save_png(),
                RibbonEvent::SaveProject => self.spawn_save_project(),
                RibbonEvent::Undo => self.undo(),
                RibbonEvent::Redo => self.redo(),
                // RibbonEvent::Settings removed - use start screen settings
            }
        }

        self.show_status_bar(ui);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(CANVAS_BG()))
            .show(ui, |ui| {
                self.show_canvas(ui);
            });

        self.show_new_dialog(ui.ctx());
        self.show_settings_dialog(ui.ctx());

        if self.show_splash {
            self.show_splash_screen(ui);
        }

        if let Some((_, t)) = &self.status
            && {
                #[cfg(target_arch = "wasm32")]
                {
                    t.elapsed_wasm() > status_ttl()
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    t.elapsed() > status_ttl()
                }
            }
        {
            self.status = None;
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let c = crate::palette::WINDOW_BG();
        [
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            1.0,
        ]
    }

fn on_exit(&mut self) {
        // Save window state
        self.egui_ctx.viewport(|vp| {
            let mut prefs = self.settings.get().clone();
            prefs.view.maximized = vp.builder.maximized.unwrap_or(false);
            prefs.view.position = vp.builder.position;
            prefs.view.size = vp.builder.inner_size;
            self.settings.update(move |p| p.view = prefs.view);
        });
        let _ = self.settings.flush();
    }
}

fn active_from_index(i: u8) -> ActiveColor {
    if i == 2 {
        ActiveColor::Two
    } else {
        ActiveColor::One
    }
}

fn non_empty_dir(s: &str) -> Option<PathBuf> {
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

fn color32_to_rgba(c: Color32) -> [u8; 4] {
    [c.r(), c.g(), c.b(), c.a()]
}

fn rgba_to_color32(p: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3])
}

/// Rasterize `text` with fontdue into a straight-alpha RGBA buffer.
fn rasterize_text(
    font: &fontdue::Font,
    text: &str,
    size: f32,
    color: [u8; 4],
) -> Option<(Vec<u8>, u32, u32)> {
    let line_height = size * 1.3;
    let lines: Vec<&str> = text.split('\n').collect();
    let mut width = 0usize;
    for line in &lines {
        let mut w = 0.0f32;
        for ch in line.chars() {
            let m = font.metrics(ch, size);
            w += m.advance_width;
        }
        width = width.max(w.ceil() as usize);
    }
    let height = ((lines.len() as f32) * line_height).ceil() as usize;
    if width == 0 || height == 0 {
        return None;
    }
    let mut buf = vec![0u8; width * height * 4];
    for (li, line) in lines.iter().enumerate() {
        let baseline = (li as f32 + 0.9) * line_height;
        let mut pen = 0.0f32;
        for ch in line.chars() {
            let (metrics, bitmap) = font.rasterize(ch, size);
            let gx = (pen + metrics.xmin as f32).round() as i32;
            // fontdue: `ymin` is the bottom edge offset measured up from the baseline.
            let top = baseline - (metrics.ymin + metrics.height as i32) as f32;
            let gy = top.round() as i32;
            for my in 0..metrics.height {
                for mx in 0..metrics.width {
                    let cov = bitmap[my * metrics.width + mx];
                    if cov == 0 {
                        continue;
                    }
                    let px = gx + mx as i32;
                    let py = gy + my as i32;
                    if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                        continue;
                    }
                    let idx = (py as usize * width + px as usize) * 4;
                    buf[idx] = color[0];
                    buf[idx + 1] = color[1];
                    buf[idx + 2] = color[2];
                    buf[idx + 3] = ((u32::from(cov) * u32::from(color[3])) / 255) as u8;
                }
            }
            pen += metrics.advance_width;
        }
    }
    Some((buf, width as u32, height as u32))
}
