//! Native `.sproj` project files: magic + dimensions + PNG payload + JSON metadata.

use serde::{Deserialize, Serialize};

const MAGIC: &[u8; 8] = b"SPAINT01";

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
pub struct ProjectMeta {
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub tool: String,
    #[serde(default = "default_stroke")]
    pub stroke: u8,
    #[serde(default)]
    pub color1: [u8; 4],
    #[serde(default)]
    pub color2: [u8; 4],
    #[serde(default)]
    pub active_color: u8,
    #[serde(default)]
    pub filled_shapes: bool,
    #[serde(default = "default_zoom")]
    pub zoom: f32,
}

fn default_stroke() -> u8 {
    8
}

fn default_zoom() -> f32 {
    1.0
}

#[derive(Debug)]
pub enum ProjectError {
    BadMagic,
    Truncated,
    BadPng(String),
    BadMeta(String),
}

impl core::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProjectError::BadMagic => write!(f, "not a SelfPaint project"),
            ProjectError::Truncated => write!(f, "file is truncated"),
            ProjectError::BadPng(e) => write!(f, "pixel data: {e}"),
            ProjectError::BadMeta(e) => write!(f, "metadata: {e}"),
        }
    }
}

/// Encode pixels + metadata into the `.sproj` container.
pub fn encode(pixels: &image::RgbaImage, meta: &ProjectMeta) -> Result<Vec<u8>, ProjectError> {
    let mut png = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png);
    image::DynamicImage::ImageRgba8(pixels.clone())
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| ProjectError::BadPng(e.to_string()))?;
    let json = serde_json::to_vec(meta).map_err(|e| ProjectError::BadMeta(e.to_string()))?;

    let mut out = Vec::with_capacity(8 + 12 + png.len() + 4 + json.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&pixels.width().to_le_bytes());
    out.extend_from_slice(&pixels.height().to_le_bytes());
    out.extend_from_slice(&(png.len() as u32).to_le_bytes());
    out.extend_from_slice(&png);
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(&json);
    Ok(out)
}

/// Decode a `.sproj` container.
pub fn decode(bytes: &[u8]) -> Result<(image::RgbaImage, ProjectMeta), ProjectError> {
    fn take<'a>(bytes: &mut &'a [u8], n: usize) -> Result<&'a [u8], ProjectError> {
        if bytes.len() < n {
            return Err(ProjectError::Truncated);
        }
        let (head, tail) = bytes.split_at(n);
        *bytes = tail;
        Ok(head)
    }

    let mut rest = bytes;
    let magic = take(&mut rest, 8)?;
    if magic != MAGIC {
        return Err(ProjectError::BadMagic);
    }
    let w = u32::from_le_bytes(take(&mut rest, 4)?.try_into().unwrap());
    let h = u32::from_le_bytes(take(&mut rest, 4)?.try_into().unwrap());
    let png_len = u32::from_le_bytes(take(&mut rest, 4)?.try_into().unwrap()) as usize;
    let png = take(&mut rest, png_len)?;
    let img = image::load_from_memory(png)
        .map_err(|e| ProjectError::BadPng(e.to_string()))?
        .into_rgba8();
    if img.dimensions() != (w, h) {
        return Err(ProjectError::BadPng("dimension mismatch".into()));
    }
    let json_len = u32::from_le_bytes(take(&mut rest, 4)?.try_into().unwrap()) as usize;
    let json = take(&mut rest, json_len)?;
    let meta: ProjectMeta =
        serde_json::from_slice(json).map_err(|e| ProjectError::BadMeta(e.to_string()))?;
    Ok((img, meta))
}

/// True when the bytes start with the project magic.
pub fn is_project(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && &bytes[..8] == MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut pixels = image::RgbaImage::new(16, 12);
        pixels.put_pixel(3, 4, image::Rgba([10, 20, 30, 255]));
        let meta = ProjectMeta {
            lang: "en".into(),
            tool: "pencil".into(),
            stroke: 4,
            color1: [1, 2, 3, 255],
            color2: [4, 5, 6, 255],
            active_color: 2,
            filled_shapes: true,
            zoom: 2.5,
        };
        let bytes = encode(&pixels, &meta).unwrap();
        assert!(is_project(&bytes));
        let (img, out_meta) = decode(&bytes).unwrap();
        assert_eq!(img.dimensions(), (16, 12));
        assert_eq!(img.get_pixel(3, 4).0, [10, 20, 30, 255]);
        assert_eq!(out_meta, meta);
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(
            decode(b"not a project at all"),
            Err(ProjectError::BadMagic)
        ));
        assert!(!is_project(b"hello"));
    }

    #[test]
    fn truncated_is_error() {
        let pixels = image::RgbaImage::new(4, 4);
        let bytes = encode(&pixels, &ProjectMeta::default()).unwrap();
        assert!(matches!(decode(&bytes[..10]), Err(ProjectError::Truncated)));
    }
}
