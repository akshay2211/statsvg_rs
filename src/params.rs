use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Profile,
    Stats,
}

impl Variant {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "stats" | "antiprofile" | "anti-profile" => Variant::Stats,
            _ => Variant::Profile,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sections {
    pub header: bool,
    pub stats: bool,
    pub grid: bool,
    pub languages: bool,
    pub top_repos: bool,
    pub contributed_to: bool,
    pub most_starred: bool,
    pub border: bool,
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub variant: Variant,
    pub width: f32,
    pub sections: Sections,
    pub top_repos_count: usize,
    pub highlight: Option<String>,
}

impl RenderConfig {
    pub fn profile() -> Self {
        Self {
            variant: Variant::Profile,
            width: 680.0,
            sections: Sections {
                header: true,
                stats: true,
                grid: true,
                languages: true,
                top_repos: true,
                contributed_to: false,
                most_starred: false,
                border: true,
            },
            top_repos_count: 3,
            highlight: None,
        }
    }

    pub fn stats() -> Self {
        Self {
            variant: Variant::Stats,
            width: 680.0,
            sections: Sections {
                header: false,
                stats: true,
                grid: false,
                languages: true,
                top_repos: false,
                contributed_to: true,
                most_starred: true,
                border: true,
            },
            top_repos_count: 3,
            highlight: None,
        }
    }

    pub fn from_variant(v: Variant) -> Self {
        match v {
            Variant::Profile => Self::profile(),
            Variant::Stats => Self::stats(),
        }
    }
}

fn parse_bool(s: &str) -> bool {
    !matches!(s.trim(), "false" | "0" | "no" | "off")
}

fn default_theme() -> String {
    "github_dark".into()
}

#[derive(Deserialize, Debug)]
pub struct RawParams {
    pub username: String,

    #[serde(default = "default_theme")]
    pub theme: String,

    pub variant: Option<String>,
    pub width: Option<u32>,
    pub highlight: Option<String>,

    pub show_header: Option<String>,
    pub show_stats: Option<String>,
    pub show_grid: Option<String>,
    pub show_languages: Option<String>,
    pub show_top_repos: Option<String>,
    pub show_contributed_to: Option<String>,
    pub show_most_starred: Option<String>,
    pub show_border: Option<String>,

    pub top_repos_count: Option<u8>,
}

impl RawParams {
    pub fn config(&self) -> RenderConfig {
        let variant = self
            .variant
            .as_deref()
            .map(Variant::parse)
            .unwrap_or(Variant::Profile);
        let mut config = RenderConfig::from_variant(variant);

        if let Some(w) = self.width {
            config.width = w.clamp(360, 1600) as f32;
        }
        if let Some(h) = &self.highlight {
            if !h.is_empty() {
                config.highlight = Some(h.clone());
            }
        }
        if let Some(c) = self.top_repos_count {
            config.top_repos_count = c.clamp(1, 6) as usize;
        }

        let s = &mut config.sections;
        if let Some(v) = &self.show_header { s.header = parse_bool(v); }
        if let Some(v) = &self.show_stats { s.stats = parse_bool(v); }
        if let Some(v) = &self.show_grid { s.grid = parse_bool(v); }
        if let Some(v) = &self.show_languages { s.languages = parse_bool(v); }
        if let Some(v) = &self.show_top_repos { s.top_repos = parse_bool(v); }
        if let Some(v) = &self.show_contributed_to { s.contributed_to = parse_bool(v); }
        if let Some(v) = &self.show_most_starred { s.most_starred = parse_bool(v); }
        if let Some(v) = &self.show_border { s.border = parse_bool(v); }

        config
    }
}
