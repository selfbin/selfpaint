//! Default color palette and Win11-light theme tokens.

use egui::{Color32, CornerRadius, Stroke};

/// The default palette grid: 4 rows × 8 columns, roughly the classic Paint set.
pub const PALETTE: [[Color32; 8]; 4] = [
    [
        Color32::from_rgb(0, 0, 0),
        Color32::from_rgb(127, 127, 127),
        Color32::from_rgb(136, 0, 21),
        Color32::from_rgb(237, 28, 36),
        Color32::from_rgb(255, 127, 39),
        Color32::from_rgb(255, 242, 0),
        Color32::from_rgb(34, 177, 76),
        Color32::from_rgb(0, 162, 232),
    ],
    [
        Color32::from_rgb(255, 255, 255),
        Color32::from_rgb(195, 195, 195),
        Color32::from_rgb(190, 190, 190),
        Color32::from_rgb(255, 192, 203),
        Color32::from_rgb(255, 202, 173), // 255,127,39 + white
        Color32::from_rgb(255, 251, 173), // yellow pastel
        Color32::from_rgb(181, 230, 181), // green pastel
        Color32::from_rgb(153, 217, 234), // blue pastel
    ],
    [
        Color32::from_rgb(255, 255, 153), // 255,242,0 + white
        Color32::from_rgb(200, 200, 75),
        Color32::from_rgb(255, 170, 140),
        Color32::from_rgb(255, 147, 129),
        Color32::from_rgb(255, 112, 186), // pink
        Color32::from_rgb(153, 84, 200),  // purple
        Color32::from_rgb(255, 64, 255),  // magenta
        Color32::from_rgb(255, 0, 128),
    ],
    [
        Color32::from_rgb(0, 128, 192), // 0,162,232 + black
        Color32::from_rgb(0, 0, 128),   // navy
        Color32::from_rgb(0, 128, 0),   // dark green
        Color32::from_rgb(128, 64, 0),  // brown
        Color32::from_rgb(128, 0, 64),  // maroon
        Color32::from_rgb(128, 128, 64),
        Color32::from_rgb(64, 128, 128),
        Color32::from_rgb(64, 64, 128),
    ],
];

// Win11 light theme tokens.
pub const WINDOW_BG: Color32 = Color32::from_rgb(0xF3, 0xF3, 0xF3);
pub const CHROME_BG: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const STATUS_BG: Color32 = Color32::from_rgb(0xF9, 0xF9, 0xF9);
pub const CANVAS_BG: Color32 = Color32::from_gray(0x76);
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0x67, 0xC0);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x00, 0x78, 0xD4);
pub const TEXT: Color32 = Color32::from_rgb(0x1A, 0x1A, 0x1A);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x60, 0x60, 0x60);
pub const BORDER: Color32 = Color32::from_rgb(0xE5, 0xE5, 0xE5);
pub const HOVER_BG: Color32 = Color32::from_rgb(0xE9, 0xE9, 0xE9);
pub const PRESSED_BG: Color32 = Color32::from_rgb(0xD0, 0xD0, 0xD0);
pub const SELECTED_BG: Color32 = Color32::from_rgb(0xD6, 0xE8, 0xF7);
pub const SELECTED_BORDER: Color32 = Color32::from_rgb(0x00, 0x67, 0xC0);

pub const CORNER: CornerRadius = CornerRadius::same(6);

/// Apply the Win11 light look to the egui visuals, including the shared
/// `craft-ui` widget palette.
pub fn apply_theme(visuals: &mut egui::Visuals) {
    visuals.window_fill = WINDOW_BG;
    visuals.panel_fill = CHROME_BG;
    visuals.extreme_bg_color = Color32::from_rgb(0xFF, 0xFF, 0xFF);
    visuals.faint_bg_color = Color32::from_rgb(0xF6, 0xF6, 0xF6);
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.hyperlink_color = ACCENT;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    let palette = craft_ui::theme::WidgetPalette {
        noninteractive: Color32::from_rgb(0xF3, 0xF3, 0xF3),
        inactive: CHROME_BG,
        hovered: HOVER_BG,
        active: PRESSED_BG,
        open: CHROME_BG,
    };
    palette.apply_to(&mut visuals.widgets, CORNER, Stroke::new(1.0, TEXT));
    // Outline-only inactive buttons, like the ribbon tools at rest.
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
}
