//! Minimal PT/EN localisation.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Pt,
    En,
}

impl Lang {
    pub fn next(self) -> Self {
        match self {
            Lang::Pt => Lang::En,
            Lang::En => Lang::Pt,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Pt => "pt",
            Lang::En => "en",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Lang::En,
            _ => Lang::Pt,
        }
    }
}

/// All user-facing strings, resolved once per language switch.
#[derive(Clone)]
pub struct L10n {
    pub new_file: &'static str,
    pub open: &'static str,
    pub save_png: &'static str,
    pub save_project: &'static str,
    pub tools_group: &'static str,
    pub shapes_group: &'static str,
    pub size_group: &'static str,
    pub text_size_group: &'static str,
    pub colors_group: &'static str,
    pub pencil: &'static str,
    pub brush: &'static str,
    pub eraser: &'static str,
    pub bucket: &'static str,
    pub text: &'static str,
    pub picker: &'static str,
    pub line: &'static str,
    pub rect: &'static str,
    pub ellipse: &'static str,
    pub filled: &'static str,
    pub font_group: &'static str,
    pub bold: &'static str,
    pub italic: &'static str,
    pub color1: &'static str,
    pub color2: &'static str,
    pub language: &'static str,
    pub zoom: &'static str,
    pub fit: &'static str,
    pub new_title: &'static str,
    pub width: &'static str,
    pub height: &'static str,
    pub ok: &'static str,
    pub cancel: &'static str,
    pub text_hint: &'static str,
    pub apply: &'static str,
    pub err_save: &'static str,
    pub saved_ok: &'static str,
    pub opened_ok: &'static str,
    pub project_saved_ok: &'static str,
    pub project_opened_ok: &'static str,
    pub untitled: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
}

impl L10n {
    pub fn of(lang: Lang) -> Self {
        match lang {
            Lang::Pt => Self {
                new_file: "Novo",
                open: "Abrir",
                save_png: "Guardar PNG",
                save_project: "Guardar projeto",
                tools_group: "Ferramentas",
                shapes_group: "Formas",
                size_group: "Espessura",
                text_size_group: "Tamanho",
                colors_group: "Cores",
                pencil: "Lápis",
                brush: "Pincel",
                eraser: "Borracha",
                bucket: "Balde de tinta",
                text: "Texto",
                picker: "Conta-gotas",
                line: "Linha",
                rect: "Retângulo",
                ellipse: "Elipse",
                filled: "Preenchido",
                font_group: "Fonte",
                bold: "Negrito",
                italic: "Itálico",
                color1: "Cor 1",
                color2: "Cor 2",
                language: "Idioma",
                zoom: "Zoom",
                fit: "Ajustar",
                new_title: "Novo documento",
                width: "Largura",
                height: "Altura",
                ok: "OK",
                cancel: "Cancelar",
                text_hint: "Clique no canvas e escreva…",
                apply: "Aplicar",
                err_save: "Falha ao guardar: {}",
                saved_ok: "Guardado: {}",
                opened_ok: "Aberto: {}",
                project_saved_ok: "Projeto guardado: {}",
                project_opened_ok: "Projeto aberto: {}",
                untitled: "Sem título",
                undo: "Desfazer (Ctrl+Z)",
                redo: "Refazer (Ctrl+Y)",
            },
            Lang::En => Self {
                new_file: "New",
                open: "Open",
                save_png: "Save PNG",
                save_project: "Save project",
                tools_group: "Tools",
                shapes_group: "Shapes",
                size_group: "Size",
                text_size_group: "Text size",
                colors_group: "Colors",
                pencil: "Pencil",
                brush: "Brush",
                eraser: "Eraser",
                bucket: "Paint bucket",
                text: "Text",
                picker: "Color picker",
                line: "Line",
                rect: "Rectangle",
                ellipse: "Ellipse",
                filled: "Filled",
                font_group: "Font",
                bold: "Bold",
                italic: "Italic",
                color1: "Color 1",
                color2: "Color 2",
                language: "Language",
                zoom: "Zoom",
                fit: "Fit",
                new_title: "New document",
                width: "Width",
                height: "Height",
                ok: "OK",
                cancel: "Cancel",
                text_hint: "Click the canvas and type…",
                apply: "Apply",
                err_save: "Failed to save: {}",
                saved_ok: "Saved: {}",
                opened_ok: "Opened: {}",
                project_saved_ok: "Project saved: {}",
                project_opened_ok: "Project opened: {}",
                untitled: "Untitled",
                undo: "Undo (Ctrl+Z)",
                redo: "Redo (Ctrl+Y)",
            },
        }
    }
}
