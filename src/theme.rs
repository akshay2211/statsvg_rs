#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub dark: bool,

    pub bg: &'static str,
    pub surface: &'static str,
    pub border: &'static str,

    pub accent_line: &'static str,

    pub text_primary: &'static str,
    pub text_muted: &'static str,
    pub text_accent: &'static str,
    pub text_streak: &'static str,

    pub grid_empty: &'static str,
    pub grid_l1: &'static str,
    pub grid_l2: &'static str,
    pub grid_l3: &'static str,
    pub grid_l4: &'static str,

    pub lang_fallbacks: &'static [&'static str],
}

impl Theme {
    pub fn grid_color(&self, count: u32) -> &'static str {
        match count {
            0 => self.grid_empty,
            1..=3 => self.grid_l1,
            4..=6 => self.grid_l2,
            7..=9 => self.grid_l3,
            _ => self.grid_l4,
        }
    }
}

pub const GITHUB_DARK: Theme = Theme {
    name: "github_dark",
    dark: true,
    bg: "#0d1117",
    surface: "#161b22",
    border: "#21262d",
    accent_line: "#7F77DD",
    text_primary: "#e6edf3",
    text_muted: "#7d8590",
    text_accent: "#AFA9EC",
    text_streak: "#39d353",
    grid_empty: "#161b22",
    grid_l1: "#0e4429",
    grid_l2: "#006d32",
    grid_l3: "#26a641",
    grid_l4: "#39d353",
    lang_fallbacks: &["#7F77DD", "#EF9F27", "#5DCAA5", "#D85A30", "#378ADD"],
};

pub const NORD: Theme = Theme {
    name: "nord",
    dark: true,
    bg: "#2e3440",
    surface: "#3b4252",
    border: "#4c566a",
    accent_line: "#88c0d0",
    text_primary: "#eceff4",
    text_muted: "#9099a8",
    text_accent: "#81a1c1",
    text_streak: "#a3be8c",
    grid_empty: "#3b4252",
    grid_l1: "#4a6741",
    grid_l2: "#5e8a50",
    grid_l3: "#78b060",
    grid_l4: "#a3be8c",
    lang_fallbacks: &["#88c0d0", "#ebcb8b", "#a3be8c", "#bf616a", "#b48ead"],
};

pub const DRACULA: Theme = Theme {
    name: "dracula",
    dark: true,
    bg: "#282a36",
    surface: "#44475a",
    border: "#6272a4",
    accent_line: "#bd93f9",
    text_primary: "#f8f8f2",
    text_muted: "#6272a4",
    text_accent: "#caa9fa",
    text_streak: "#50fa7b",
    grid_empty: "#44475a",
    grid_l1: "#1e5e31",
    grid_l2: "#28803f",
    grid_l3: "#38b85b",
    grid_l4: "#50fa7b",
    lang_fallbacks: &["#bd93f9", "#ffb86c", "#50fa7b", "#ff5555", "#8be9fd"],
};

pub const LIGHT_CLEAN: Theme = Theme {
    name: "light",
    dark: false,
    bg: "#ffffff",
    surface: "#f3f4f6",
    border: "#e5e7eb",
    accent_line: "#7F77DD",
    text_primary: "#111827",
    text_muted: "#6b7280",
    text_accent: "#534AB7",
    text_streak: "#16a34a",
    grid_empty: "#f0fdf4",
    grid_l1: "#dcfce7",
    grid_l2: "#86efac",
    grid_l3: "#22c55e",
    grid_l4: "#16a34a",
    lang_fallbacks: &["#7F77DD", "#d97706", "#16a34a", "#dc2626", "#2563eb"],
};

pub const SOLARIZED: Theme = Theme {
    name: "solarized",
    dark: false,
    bg: "#fdf6e3",
    surface: "#eee8d5",
    border: "#93a1a1",
    accent_line: "#268bd2",
    text_primary: "#073642",
    text_muted: "#657b83",
    text_accent: "#268bd2",
    text_streak: "#859900",
    grid_empty: "#eee8d5",
    grid_l1: "#3d5900",
    grid_l2: "#556000",
    grid_l3: "#6d7c00",
    grid_l4: "#859900",
    lang_fallbacks: &["#268bd2", "#cb4b16", "#859900", "#dc322f", "#6c71c4"],
};

pub const ALL_THEMES: &[&Theme] = &[&GITHUB_DARK, &NORD, &DRACULA, &LIGHT_CLEAN, &SOLARIZED];

pub fn find_theme(name: &str) -> &'static Theme {
    ALL_THEMES
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
        .copied()
        .unwrap_or(&GITHUB_DARK)
}