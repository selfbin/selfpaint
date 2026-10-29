//! Colour themes (light / dark / gray / black / Breeze) and the swatch palette.

use std::sync::atomic::{AtomicU8, Ordering};

use egui::{Color32, CornerRadius, Stroke};

/// The active colour theme.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    Gray,
    Black,
    BreezeLight,
    BreezeDark,
}

/// One theme's colour tokens.
struct Tokens {
    dark: bool,
    window_bg: Color32,
    chrome_bg: Color32,
    status_bg: Color32,
    canvas_bg: Color32,
    accent: Color32,
    accent_hover: Color32,
    text: Color32,
    text_muted: Color32,
    border: Color32,
    hover_bg: Color32,
    pressed_bg: Color32,
    selected_bg: Color32,
    selected_border: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

const LIGHT: Tokens = Tokens {
    dark: false,
    window_bg: rgb(0xEE, 0xEE, 0xEE),
    chrome_bg: rgb(0xFF, 0xFF, 0xFF),
    status_bg: rgb(0xF5, 0xF5, 0xF5),
    canvas_bg: rgb(0xEE, 0xEE, 0xEE),
    accent: rgb(0x3D, 0x0B, 0x7C),
    accent_hover: rgb(0x5A, 0x1C, 0xAB),
    text: rgb(0x1A, 0x1A, 0x1A),
    text_muted: rgb(0x6B, 0x6B, 0x6B),
    border: rgb(0xD0, 0xD0, 0xD0),
    hover_bg: rgb(0xE8, 0xE0, 0xF8),
    pressed_bg: rgb(0xD0, 0xC0, 0xE0),
    selected_bg: rgb(0xE8, 0xE0, 0xF8),
    selected_border: rgb(0x3D, 0x0B, 0x7C),
};

const DARK: Tokens = Tokens {
    dark: true,
    window_bg: rgb(0x20, 0x20, 0x20),
    chrome_bg: rgb(0x2B, 0x2B, 0x2B),
    status_bg: rgb(0x27, 0x27, 0x27),
    canvas_bg: rgb(0x1E, 0x1E, 0x1E),
    accent: rgb(0xB5, 0x8F, 0xFB),
    accent_hover: rgb(0xC9, 0xAA, 0xFF),
    text: rgb(0xEE, 0xEE, 0xEE),
    text_muted: rgb(0xA0, 0xA0, 0xA0),
    border: rgb(0x3A, 0x3A, 0x3A),
    hover_bg: rgb(0x33, 0x2A, 0x47),
    pressed_bg: rgb(0x2A, 0x22, 0x3A),
    selected_bg: rgb(0x2B, 0x1A, 0x52),
    selected_border: rgb(0xB5, 0x8F, 0xFB),
};

const GRAY: Tokens = Tokens {
    dark: true,
    window_bg: rgb(0x2E, 0x2E, 0x2E),
    chrome_bg: rgb(0x38, 0x38, 0x38),
    status_bg: rgb(0x33, 0x33, 0x33),
    canvas_bg: rgb(0x22, 0x22, 0x22),
    accent: rgb(0xB5, 0x8F, 0xFB),
    accent_hover: rgb(0xC9, 0xAA, 0xFF),
    text: rgb(0xE8, 0xE8, 0xE8),
    text_muted: rgb(0x9A, 0x9A, 0x9A),
    border: rgb(0x4A, 0x4A, 0x4A),
    hover_bg: rgb(0x3D, 0x34, 0x52),
    pressed_bg: rgb(0x33, 0x2B, 0x46),
    selected_bg: rgb(0x2B, 0x1A, 0x52),
    selected_border: rgb(0xB5, 0x8F, 0xFB),
};

const BLACK: Tokens = Tokens {
    dark: true,
    window_bg: rgb(0x0A, 0x0A, 0x0A),
    chrome_bg: rgb(0x12, 0x12, 0x12),
    status_bg: rgb(0x0E, 0x0E, 0x0E),
    canvas_bg: rgb(0x00, 0x00, 0x00),
    accent: rgb(0xB5, 0x8F, 0xFB),
    accent_hover: rgb(0xC9, 0xAA, 0xFF),
    text: rgb(0xEE, 0xEE, 0xEE),
    text_muted: rgb(0x80, 0x80, 0x80),
    border: rgb(0x2A, 0x2A, 0x2A),
    hover_bg: rgb(0x24, 0x1B, 0x3A),
    pressed_bg: rgb(0x1C, 0x14, 0x2D),
    selected_bg: rgb(0x24, 0x17, 0x45),
    selected_border: rgb(0xB5, 0x8F, 0xFB),
};

const BREEZE_LIGHT: Tokens = Tokens {
    dark: false,
    window_bg: rgb(0xEF, 0xF0, 0xF1),
    chrome_bg: rgb(0xFC, 0xFC, 0xFC),
    status_bg: rgb(0xEF, 0xF0, 0xF1),
    canvas_bg: rgb(0xEE, 0xEE, 0xEE),
    accent: rgb(0x3D, 0x0B, 0x7C),
    accent_hover: rgb(0x5A, 0x1C, 0xAB),
    text: rgb(0x23, 0x26, 0x29),
    text_muted: rgb(0x7A, 0x7E, 0x83),
    border: rgb(0xD5, 0xD8, 0xDB),
    hover_bg: rgb(0xE8, 0xE0, 0xF8),
    pressed_bg: rgb(0xD0, 0xC0, 0xE0),
    selected_bg: rgb(0xE8, 0xE0, 0xF8),
    selected_border: rgb(0x3D, 0x0B, 0x7C),
};

const BREEZE_DARK: Tokens = Tokens {
    dark: true,
    window_bg: rgb(0x23, 0x26, 0x29),
    chrome_bg: rgb(0x2A, 0x2E, 0x32),
    status_bg: rgb(0x23, 0x26, 0x29),
    canvas_bg: rgb(0x1B, 0x1E, 0x20),
    accent: rgb(0xB5, 0x8F, 0xFB),
    accent_hover: rgb(0xC9, 0xAA, 0xFF),
    text: rgb(0xEF, 0xF0, 0xF1),
    text_muted: rgb(0x9D, 0xA3, 0xA8),
    border: rgb(0x3A, 0x3F, 0x44),
    hover_bg: rgb(0x33, 0x2A, 0x47),
    pressed_bg: rgb(0x2A, 0x22, 0x3A),
    selected_bg: rgb(0x2B, 0x1A, 0x52),
    selected_border: rgb(0xB5, 0x8F, 0xFB),
};

impl Theme {
    pub const ALL: [Theme; 6] = [
        Theme::Light,
        Theme::Dark,
        Theme::Gray,
        Theme::Black,
        Theme::BreezeLight,
        Theme::BreezeDark,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::Gray => "gray",
            Theme::Black => "black",
            Theme::BreezeLight => "breeze-light",
            Theme::BreezeDark => "breeze-dark",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "dark" => Theme::Dark,
            "gray" => Theme::Gray,
            "black" => Theme::Black,
            "breeze-light" => Theme::BreezeLight,
            "breeze-dark" => Theme::BreezeDark,
            _ => Theme::Light,
        }
    }

    pub fn is_dark(self) -> bool {
        self.tokens().dark
    }

    fn tokens(self) -> &'static Tokens {
        match self {
            Theme::Light => &LIGHT,
            Theme::Dark => &DARK,
            Theme::Gray => &GRAY,
            Theme::Black => &BLACK,
            Theme::BreezeLight => &BREEZE_LIGHT,
            Theme::BreezeDark => &BREEZE_DARK,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set_current(theme: Theme) {
    CURRENT.store(theme as u8, Ordering::Relaxed);
}

pub fn current() -> Theme {
    Theme::ALL
        .get(CURRENT.load(Ordering::Relaxed) as usize)
        .copied()
        .unwrap_or(Theme::Light)
}

fn t() -> &'static Tokens {
    current().tokens()
}

// The uppercase names are accessor functions so every call site picks up the
// active theme.
#[allow(non_snake_case)]
pub fn WINDOW_BG() -> Color32 {
    t().window_bg
}
#[allow(non_snake_case)]
pub fn CHROME_BG() -> Color32 {
    t().chrome_bg
}
#[allow(non_snake_case)]
pub fn STATUS_BG() -> Color32 {
    t().status_bg
}
#[allow(non_snake_case)]
pub fn CANVAS_BG() -> Color32 {
    t().canvas_bg
}
#[allow(non_snake_case)]
pub fn ACCENT() -> Color32 {
    t().accent
}
#[allow(non_snake_case)]
pub fn ACCENT_HOVER() -> Color32 {
    t().accent_hover
}
#[allow(non_snake_case)]
pub fn TEXT() -> Color32 {
    t().text
}
#[allow(non_snake_case)]
pub fn TEXT_MUTED() -> Color32 {
    t().text_muted
}
#[allow(non_snake_case)]
pub fn BORDER() -> Color32 {
    t().border
}
#[allow(non_snake_case)]
pub fn HOVER_BG() -> Color32 {
    t().hover_bg
}
#[allow(non_snake_case)]
pub fn PRESSED_BG() -> Color32 {
    t().pressed_bg
}
#[allow(non_snake_case)]
pub fn SELECTED_BG() -> Color32 {
    t().selected_bg
}
#[allow(non_snake_case)]
pub fn SELECTED_BORDER() -> Color32 {
    t().selected_border
}

pub const CORNER: CornerRadius = CornerRadius::same(6);

/// The swatch grid offered in the ribbon: 16 colours in two rows.
pub const PALETTE: [[Color32; 8]; 2] = [
    [
        Color32::from_rgb(0x00, 0x00, 0x00),
        Color32::from_rgb(0x7F, 0x7F, 0x7F),
        Color32::from_rgb(0x88, 0x0F, 0x1B),
        Color32::from_rgb(0xE8, 0x11, 0x23),
        Color32::from_rgb(0xFF, 0x8C, 0x00),
        Color32::from_rgb(0xFF, 0xD4, 0x00),
        Color32::from_rgb(0x10, 0x7C, 0x10),
        Color32::from_rgb(0x00, 0x6F, 0xC8),
    ],
    [
        Color32::from_rgb(0xFF, 0xFF, 0xFF),
        Color32::from_rgb(0xC8, 0xC8, 0xC8),
        Color32::from_rgb(0x99, 0x4D, 0x00),
        Color32::from_rgb(0xFF, 0x8A, 0x9B),
        Color32::from_rgb(0xFF, 0xB0, 0x70),
        Color32::from_rgb(0xFF, 0xF4, 0x9B),
        Color32::from_rgb(0x9B, 0xD6, 0x9B),
        Color32::from_rgb(0x9B, 0xDC, 0xE8),
    ],
];

/// Apply the theme to the egui visuals, including the shared `craft-ui`
/// widget palette.
pub fn apply_theme(visuals: &mut egui::Visuals, theme: Theme) {
    *visuals = if theme.is_dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.window_fill = WINDOW_BG();
    visuals.panel_fill = CHROME_BG();
    visuals.extreme_bg_color = CHROME_BG();
    visuals.faint_bg_color = HOVER_BG();
    visuals.selection.bg_fill = ACCENT();
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.hyperlink_color = ACCENT();
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER());
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER());
    visuals.window_stroke = Stroke::new(1.0, BORDER());
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    let palette = craft_ui::theme::WidgetPalette {
        noninteractive: WINDOW_BG(),
        inactive: CHROME_BG(),
        hovered: HOVER_BG(),
        active: PRESSED_BG(),
        open: CHROME_BG(),
    };
    palette.apply_to(&mut visuals.widgets, CORNER, Stroke::new(1.0, TEXT()));
    // Outline-only inactive buttons, like the ribbon tools at rest.
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
}
