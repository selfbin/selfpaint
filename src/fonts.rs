//! Font catalogue for the text tool: bundled OFL faces plus system fonts.
//!
//! Families are grouped by family name; each may carry Regular / Bold / Italic /
//! Bold-Italic faces. Bundled fonts are always available; system fonts are
//! discovered once at startup by scanning the usual Linux font directories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Clone)]
enum Source {
    Embedded(&'static [u8]),
    Path(PathBuf),
}

#[derive(Default)]
struct FamilyFaces {
    regular: Option<Source>,
    bold: Option<Source>,
    italic: Option<Source>,
    bold_italic: Option<Source>,
}

impl FamilyFaces {
    fn get(&self, style: FontStyle) -> Option<&Source> {
        match style {
            FontStyle::Regular => self.regular.as_ref(),
            FontStyle::Bold => self.bold.as_ref().or(self.regular.as_ref()),
            FontStyle::Italic => self.italic.as_ref().or(self.regular.as_ref()),
            FontStyle::BoldItalic => self
                .bold_italic
                .as_ref()
                .or(self.bold.as_ref())
                .or(self.italic.as_ref())
                .or(self.regular.as_ref()),
        }
    }

    fn set(&mut self, style: FontStyle, source: Source) {
        let slot = match style {
            FontStyle::Regular => &mut self.regular,
            FontStyle::Bold => &mut self.bold,
            FontStyle::Italic => &mut self.italic,
            FontStyle::BoldItalic => &mut self.bold_italic,
        };
        // First face wins for a given style; embedded fonts are inserted first.
        if slot.is_none() {
            *slot = Some(source);
        }
    }
}

pub struct FontCatalog {
    /// Family names in display order (bundled first, then alphabetical).
    families: Vec<String>,
    faces: HashMap<String, FamilyFaces>,
}

impl Default for FontCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl FontCatalog {
    pub fn new() -> Self {
        let mut catalog = Self {
            families: Vec::new(),
            faces: HashMap::new(),
        };
        catalog.add_embedded();
        catalog.scan_system();
        catalog
    }

    /// Family names for the picker, in display order.
    pub fn families(&self) -> &[String] {
        &self.families
    }

    pub fn family_index(&self, name: &str) -> usize {
        self.families.iter().position(|f| f == name).unwrap_or(0)
    }

    /// Raw bytes of a face, if the family has one (with style fallbacks).
    pub fn bytes(&self, family: &str, style: FontStyle) -> Option<CowBytes<'_>> {
        let faces = self.faces.get(family)?;
        match faces.get(style)? {
            Source::Embedded(b) => Some(CowBytes::Borrowed(b)),
            Source::Path(p) => std::fs::read(p).ok().map(CowBytes::Owned),
        }
    }

    /// Display name of a family by index, if in range.
    pub fn family_name(&self, index: usize) -> Option<&str> {
        self.families.get(index).map(String::as_str)
    }

    /// Load a `fontdue` face for rasterization.
    pub fn load(&self, family: &str, style: FontStyle) -> Option<fontdue::Font> {
        let bytes = self.bytes(family, style)?;
        fontdue::Font::from_bytes(bytes.as_ref(), fontdue::FontSettings::default()).ok()
    }

    fn add_family(&mut self, name: &str, style: FontStyle, source: Source) {
        if !self.faces.contains_key(name) {
            self.faces.insert(name.to_owned(), FamilyFaces::default());
            self.families.push(name.to_owned());
        }
        self.faces.get_mut(name).unwrap().set(style, source);
    }

    fn add_embedded(&mut self) {
        macro_rules! embed {
            ($family:literal, $style:expr, $path:literal) => {
                self.add_family(
                    $family,
                    $style,
                    Source::Embedded(include_bytes!(concat!("../assets/fonts/", $path))),
                );
            };
        }
        embed!("Open Sans", FontStyle::Regular, "OpenSans-Regular.ttf");
        embed!("Open Sans", FontStyle::Bold, "OpenSans-Bold.ttf");
        embed!("Open Sans", FontStyle::Italic, "OpenSans-Italic.ttf");
        embed!(
            "Open Sans",
            FontStyle::BoldItalic,
            "OpenSans-BoldItalic.ttf"
        );
        embed!(
            "JetBrains Mono",
            FontStyle::Regular,
            "JetBrainsMono-Regular.ttf"
        );
        embed!("JetBrains Mono", FontStyle::Bold, "JetBrainsMono-Bold.ttf");
        embed!(
            "JetBrains Mono",
            FontStyle::Italic,
            "JetBrainsMono-Italic.ttf"
        );
        embed!("Pacifico", FontStyle::Regular, "Pacifico-Regular.ttf");
    }

    fn scan_system(&mut self) {
        let embedded_count = self.families.len();
        let mut dirs: Vec<PathBuf> = SYSTEM_FONT_DIRS.iter().map(PathBuf::from).collect();
        dirs.extend(user_font_dirs());
        for dir in dirs {
            if dir.is_dir() {
                self.scan_dir(&dir, 0);
            }
        }
        // Keep bundled families first; sort the rest alphabetically.
        self.families[embedded_count..].sort();
    }

    fn scan_dir(&mut self, dir: &Path, depth: u32) {
        if depth > 6 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_dir(&path, depth + 1);
                continue;
            }
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if !ext.eq_ignore_ascii_case("ttf") && !ext.eq_ignore_ascii_case("otf") {
                continue;
            }
            let Ok(data) = std::fs::read(&path) else {
                continue;
            };
            if let Some((family, style)) = face_info(&data) {
                self.add_family(&family, style, Source::Path(path));
            }
        }
    }
}

pub enum CowBytes<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl AsRef<[u8]> for CowBytes<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            CowBytes::Borrowed(b) => b,
            CowBytes::Owned(b) => b,
        }
    }
}

const SYSTEM_FONT_DIRS: &[&str] = &[
    "/usr/share/fonts",
    "/usr/local/share/fonts",
    "/usr/share/fonts/truetype",
    "/usr/share/fonts/opentype",
];

fn user_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(&home).join(".fonts"));
        dirs.push(PathBuf::from(&home).join(".local/share/fonts"));
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(xdg).join("fonts"));
        }
    }
    if let Ok(data) = std::env::var("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data).join("fonts"));
    }
    dirs
}

/// Extract (family, style) from a TTF/OTF file by reading its `name`, `head`
/// and `OS/2` tables. Minimal big-endian parser; no external dependency.
fn face_info(data: &[u8]) -> Option<(String, FontStyle)> {
    if data.len() < 12 {
        return None;
    }
    let magic = u32::from_be_bytes(data[0..4].try_into().ok()?);
    // 0x00010000 (TrueType), 'OTTO' (CFF), 'true', 'ttcf' (collection -> first face).
    if magic == 0x7474_6366 {
        // 'ttcf' — offset of first font is at byte 12.
        return None;
    }
    if magic != 0x0001_0000 && magic != 0x4F54_544F && magic != 0x7472_7565 {
        return None;
    }
    let num_tables = u16::from_be_bytes(data[4..6].try_into().ok()?) as usize;

    let mut name_table: Option<(usize, usize)> = None;
    let mut head_table: Option<usize> = None;
    let mut os2_table: Option<usize> = None;
    for i in 0..num_tables {
        let rec = 12 + i * 16;
        if data.len() < rec + 16 {
            return None;
        }
        let tag = &data[rec..rec + 4];
        let offset = u32::from_be_bytes(data[rec + 8..rec + 12].try_into().ok()?) as usize;
        let length = u32::from_be_bytes(data[rec + 12..rec + 16].try_into().ok()?) as usize;
        match tag {
            b"name" => name_table = Some((offset, length)),
            b"head" => head_table = Some(offset),
            b"OS/2" => os2_table = Some(offset),
            _ => {}
        }
    }

    let (name_off, name_len) = name_table?;
    if data.len() < name_off + name_len || name_len < 6 {
        return None;
    }
    let name = &data[name_off..name_off + name_len];
    let count = u16::from_be_bytes(name[2..4].try_into().ok()?) as usize;
    let strings_off = u16::from_be_bytes(name[4..6].try_into().ok()?) as usize;

    let mut family: Option<String> = None;
    let mut typographic_family: Option<String> = None;
    for i in 0..count {
        let rec = 6 + i * 12;
        if name.len() < rec + 12 {
            break;
        }
        let platform = u16::from_be_bytes(name[rec..rec + 2].try_into().ok()?);
        let encoding = u16::from_be_bytes(name[rec + 2..rec + 4].try_into().ok()?);
        let name_id = u16::from_be_bytes(name[rec + 6..rec + 8].try_into().ok()?);
        let length = u16::from_be_bytes(name[rec + 8..rec + 10].try_into().ok()?) as usize;
        let offset = u16::from_be_bytes(name[rec + 10..rec + 12].try_into().ok()?) as usize;
        if name_id != 1 && name_id != 16 {
            continue;
        }
        // Prefer Windows Unicode (platform 3, encoding 1) or any Unicode platform.
        let unicode = platform == 3 && encoding == 1 || platform == 0;
        if !unicode {
            continue;
        }
        let start = strings_off + offset;
        let end = start + length;
        if end > name.len() {
            continue;
        }
        let raw = &name[start..end];
        let text = decode_utf16_be(raw);
        if text.is_empty() {
            continue;
        }
        match name_id {
            16 => {
                typographic_family.get_or_insert(text);
            }
            _ => {
                family.get_or_insert(text);
            }
        }
    }
    let family = typographic_family.or(family)?;

    // Style from head.macStyle (bit 0 = bold, bit 1 = italic) and OS/2.usWeightClass.
    let mut bold = false;
    let mut italic = false;
    if let Some(off) = head_table
        && data.len() >= off + 46
    {
        let mac_style = u16::from_be_bytes(data[off + 44..off + 46].try_into().ok()?);
        bold |= mac_style & 1 != 0;
        italic |= mac_style & 2 != 0;
    }
    if let Some(off) = os2_table
        && data.len() >= off + 6
    {
        let weight = u16::from_be_bytes(data[off + 4..off + 6].try_into().ok()?);
        bold |= weight >= 700;
    }
    let style = match (bold, italic) {
        (true, true) => FontStyle::BoldItalic,
        (true, false) => FontStyle::Bold,
        (false, true) => FontStyle::Italic,
        (false, false) => FontStyle::Regular,
    };
    Some((family, style))
}

fn decode_utf16_be(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes(*c))
        .collect();
    String::from_utf16(&units).unwrap_or_default()
}

/// Style implied by the bold/italic toggles.
pub fn style_from_flags(bold: bool, italic: bool) -> FontStyle {
    match (bold, italic) {
        (true, true) => FontStyle::BoldItalic,
        (true, false) => FontStyle::Bold,
        (false, true) => FontStyle::Italic,
        (false, false) => FontStyle::Regular,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_families_present() {
        let catalog = FontCatalog::new();
        for name in ["Open Sans", "JetBrains Mono", "Pacifico"] {
            assert!(
                catalog.families().iter().any(|f| f == name),
                "missing {name}"
            );
            assert!(catalog.bytes(name, FontStyle::Regular).is_some());
        }
    }

    #[test]
    fn style_fallbacks() {
        let catalog = FontCatalog::new();
        // Pacifico has only Regular; Bold must fall back to it.
        assert!(catalog.bytes("Pacifico", FontStyle::Bold).is_some());
        assert!(catalog.bytes("Pacifico", FontStyle::BoldItalic).is_some());
    }

    #[test]
    fn family_index_defaults_to_zero() {
        let catalog = FontCatalog::new();
        assert_eq!(catalog.family_index("no-such-font"), 0);
    }

    #[test]
    fn loads_fontdue_face() {
        let catalog = FontCatalog::new();
        let font = catalog.load("Open Sans", FontStyle::Regular).unwrap();
        assert!(font.chars().contains_key(&'A'));
    }

    #[test]
    fn parses_embedded_open_sans_name_table() {
        let data = include_bytes!("../assets/fonts/OpenSans-Regular.ttf");
        let (family, style) = face_info(data).unwrap();
        assert_eq!(family, "Open Sans");
        assert_eq!(style, FontStyle::Regular);
    }

    #[test]
    fn parses_bold_and_italic() {
        let bold = include_bytes!("../assets/fonts/OpenSans-Bold.ttf");
        assert_eq!(
            face_info(bold).unwrap(),
            ("Open Sans".into(), FontStyle::Bold)
        );
        let italic = include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf");
        assert_eq!(
            face_info(italic).unwrap(),
            ("JetBrains Mono".into(), FontStyle::Italic)
        );
    }
}
