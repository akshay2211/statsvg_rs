use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Sections {
    pub header: bool,
    pub stats: bool,
    pub grid: bool,
    pub languages: bool,
    pub top_repos: bool,
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

    pub show_header: Option<String>,
    pub show_stats: Option<String>,
    pub show_grid: Option<String>,
    pub show_languages: Option<String>,
    pub show_top_repos: Option<String>,

    pub top_repos_count: Option<u8>,
}

impl RawParams {
    pub fn sections(&self) -> Sections {
        Sections {
            header: self.show_header.as_deref().map(parse_bool).unwrap_or(true),
            stats: self.show_stats.as_deref().map(parse_bool).unwrap_or(true),
            grid: self.show_grid.as_deref().map(parse_bool).unwrap_or(true),
            languages: self.show_languages.as_deref().map(parse_bool).unwrap_or(true),
            top_repos: self.show_top_repos.as_deref().map(parse_bool).unwrap_or(false),
        }
    }

    pub fn top_repos_count(&self) -> usize {
        self.top_repos_count
            .map(|n| n.clamp(1, 6) as usize)
            .unwrap_or(3)
    }
}