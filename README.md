# SelfPaint

A modern Windows 11 Paint clone for Linux, built with Rust, [eframe/egui](https://github.com/emilk/egui) and the [craft](https://github.com/storytold/craft-libs) UI/settings libraries.

## Features

- **Win11-style ribbon**: tools, shapes, stroke size, filled-shape toggle, Cor 1 / Cor 2 color indicators and a 4×8 palette grid
- **Tools**: pencil, brush, eraser, paint bucket (flood fill), text, color picker, line, rectangle, ellipse
- **Canvas**: gray workspace with a white sheet, smooth zoom (Ctrl+scroll) and pan (middle-drag), fit-to-window
- **Status bar**: document dimensions, transient messages, zoom slider with fit button
- **Undo/redo** (Ctrl+Z / Ctrl+Y or Ctrl+Shift+Z) with tight bounding-box history
- **Files**: New, Open (PNG/JPEG/`.sproj`), Save PNG, Save project — all dialogs run off the UI thread
- **`.sproj` project format**: magic header + PNG pixels + JSON metadata (tool, colors, zoom, language…)
- **i18n**: Portuguese and English, switchable from the ribbon and persisted
- **Light Win11 theme** with shared `craft-ui` widget palette

## Building

```sh
cargo build --release
./target/release/selfpaint
```

Requires a Linux desktop with OpenGL (any modern X11/Wayland session). System libraries needed by `rfd` (native file dialogs): `libgtk-3-dev` (or equivalent) at build time.

## Tests

```sh
cargo test
```

## Project layout

| File | Role |
|---|---|
| `src/app.rs` | Application shell: panels, canvas interaction, file jobs |
| `src/ribbon.rs` | Top ribbon UI (craft-ui icon buttons) |
| `src/canvas.rs` | View transform, sheet/overlay painting |
| `src/document.rs` | Pixel document: stamps, strokes, flood fill, shapes |
| `src/history.rs` | Undo/redo via bounding-box pixel diffs |
| `src/project.rs` | `.sproj` encode/decode |
| `src/tools.rs` | Tool enum + shape preview state |
| `src/palette.rs` | Default palette + Win11 light theme tokens |
| `src/i18n.rs` | PT/EN strings |
| `src/settings.rs` | Persistent preferences (craft-settings) |

## License

MIT
