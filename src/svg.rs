use std::fmt::Write;

use base64::Engine;
use chrono::Datelike;

use crate::github::GitHubStats;
use crate::params::{RenderConfig, Variant};
use crate::theme::Theme;

const PAD: f32 = 32.0;

struct Builder<'a> {
    buf: String,
    theme: &'a Theme,
    cursor_y: f32,
    width: f32,
}

impl<'a> Builder<'a> {
    fn new(theme: &'a Theme, width: f32) -> Self {
        Self {
            buf: String::with_capacity(8192),
            theme,
            cursor_y: 0.0,
            width,
        }
    }

    fn divider(&mut self) {
        let y = self.cursor_y;
        let _ = write!(
            self.buf,
            r#"<line x1="{:.1}" x2="{:.1}" y1="{y:.1}" y2="{y:.1}" stroke="{}" stroke-width="1"/>"#,
            PAD,
            self.width - PAD,
            self.theme.border
        );
    }

    fn section_label(&mut self, text: &str, x: f32, extra_y: f32) {
        let y = self.cursor_y + extra_y;
        let _ = write!(
            self.buf,
            r#"<text x="{x:.1}" y="{y:.1}" font-size="9" font-family="sans-serif" fill="{}" letter-spacing="1">{}</text>"#,
            self.theme.text_muted,
            esc(text)
        );
    }
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn fmt_k(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

fn lang_color<'a>(color: &'a str, idx: usize, theme: &'a Theme) -> &'a str {
    if color.starts_with('#') {
        color
    } else {
        theme.lang_fallbacks[idx % theme.lang_fallbacks.len()]
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        let take = max.saturating_sub(1);
        let mut out: String = s.chars().take(take).collect();
        out.push('…');
        out
    }
}

fn wrap_two_lines(text: &str, line_chars: usize) -> Vec<String> {
    let line_chars = line_chars.max(8);
    if text.is_empty() {
        return vec![];
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= line_chars {
        return vec![text.to_string()];
    }
    let first_end = find_break(&chars, line_chars);
    let first: String = chars[..first_end].iter().collect();
    let rest: String = chars[first_end..]
        .iter()
        .collect::<String>()
        .trim_start()
        .to_string();
    if rest.chars().count() <= line_chars {
        return vec![first.trim_end().to_string(), rest];
    }
    let second = truncate_chars(&rest, line_chars);
    vec![first.trim_end().to_string(), second]
}

fn find_break(chars: &[char], max: usize) -> usize {
    if chars.len() <= max {
        return chars.len();
    }
    let lower = max.saturating_sub(12);
    for i in (lower..=max).rev() {
        if i < chars.len() && chars[i].is_whitespace() {
            return i;
        }
    }
    max
}

fn render_header(b: &mut Builder, stats: &GitHubStats) {
    let y0 = b.cursor_y;
    let theme = b.theme;
    let card_w = b.width;

    if !stats.avatar_bytes.is_empty() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&stats.avatar_bytes);
        let mime = if stats.avatar_mime.is_empty() {
            "image/png"
        } else {
            stats.avatar_mime.as_str()
        };
        let _ = write!(
            b.buf,
            r#"<defs><clipPath id="avatar-clip"><circle cx="52" cy="{:.1}" r="28"/></clipPath></defs>"#,
            y0 + 50.0
        );
        let _ = write!(
            b.buf,
            r#"<image href="data:{mime};base64,{b64}" x="24" y="{:.1}" width="56" height="56" clip-path="url(#avatar-clip)" preserveAspectRatio="xMidYMid slice"/>"#,
            y0 + 22.0
        );
    } else {
        let avatar_cy = y0 + 50.0;
        let _ = write!(
            b.buf,
            r#"<circle cx="52" cy="{avatar_cy:.1}" r="28" fill="{}"/>"#,
            theme.surface
        );
        let first = stats
            .name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        let _ = write!(
            b.buf,
            r#"<text x="52" y="{avatar_cy:.1}" font-family="monospace" font-size="32" font-weight="600" fill="{}" text-anchor="middle" dominant-baseline="central">{}</text>"#,
            theme.accent_line,
            esc(&first)
        );
    }

    let _ = write!(
        b.buf,
        r#"<text x="92" y="{:.1}" font-size="15" font-weight="500" font-family="sans-serif" fill="{}">{}</text>"#,
        y0 + 32.0,
        theme.text_primary,
        esc(&stats.name)
    );

    let _ = write!(
        b.buf,
        r#"<text x="92" y="{:.1}" font-size="11" font-family="sans-serif" fill="{}">@{}</text>"#,
        y0 + 48.0,
        theme.text_muted,
        esc(&stats.login)
    );

    if !stats.bio.is_empty() {
        let bio = truncate_chars(&stats.bio, 55);
        let _ = write!(
            b.buf,
            r#"<text x="92" y="{:.1}" font-size="11" font-family="sans-serif" fill="{}">{}</text>"#,
            y0 + 66.0,
            theme.text_muted,
            esc(&bio)
        );
    }

    if !stats.location.is_empty() {
        let _ = write!(
            b.buf,
            r#"<text x="92" y="{:.1}" font-size="11" font-family="sans-serif" fill="{}">{}</text>"#,
            y0 + 84.0,
            theme.text_muted,
            esc(&stats.location)
        );
    }

    let pill_w = 168.0_f32;
    let pill_x = card_w - PAD - pill_w;
    let pill_y = y0 + 32.0;
    let _ = write!(
        b.buf,
        r#"<rect x="{pill_x:.1}" y="{pill_y:.1}" width="{pill_w:.1}" height="22" rx="11" fill="{}" stroke="{}" stroke-width="0.5"/>"#,
        theme.surface, theme.accent_line
    );
    let pill_text = format!(
        "{} followers · {} following",
        fmt_k(stats.followers),
        fmt_k(stats.following)
    );
    let _ = write!(
        b.buf,
        r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}" text-anchor="middle">{}</text>"#,
        pill_x + pill_w / 2.0,
        pill_y + 14.0,
        theme.text_accent,
        esc(&pill_text)
    );

    let years = (chrono::Utc::now().year() - stats.created_year).max(0);
    let _ = write!(
        b.buf,
        r#"<text x="{:.1}" y="{:.1}" font-size="9" font-family="sans-serif" fill="{}" text-anchor="end">Member since {} · {} years on GitHub</text>"#,
        card_w - PAD,
        pill_y + 38.0,
        theme.text_muted,
        stats.created_year,
        years
    );

    b.cursor_y += 104.0;
}

fn render_stats_cells(b: &mut Builder, stats: &GitHubStats, variant: Variant) {
    b.divider();
    b.cursor_y += 14.0;
    let theme = b.theme;
    let card_w = b.width;

    let cells: [(String, &str, bool); 6] = match variant {
        Variant::Profile => [
            (fmt_k(stats.total_contributions), "contributions", false),
            (format!("{} ★", stats.current_streak), "day streak", true),
            (fmt_k(stats.total_commits), "commits", false),
            (fmt_k(stats.total_stars), "total stars", false),
            (fmt_k(stats.total_prs), "PRs merged", false),
            (fmt_k(stats.total_forks), "forks", false),
        ],
        Variant::Stats => [
            (fmt_k(stats.lifetime_contributions), "lifetime contributions", false),
            (format!("{} ★", stats.longest_streak), "longest streak", true),
            (fmt_k(stats.lifetime_commits), "lifetime commits", false),
            (fmt_k(stats.total_stars), "total stars", false),
            (fmt_k(stats.lifetime_prs), "lifetime PRs", false),
            (fmt_k(stats.total_forks), "forks across repos", false),
        ],
    };

    let n = cells.len() as f32;
    let gap = 8.0;
    let cell_w = (card_w - PAD * 2.0 - gap * (n - 1.0)) / n;
    let cell_h = 52.0;
    let y = b.cursor_y;

    for (i, (val, label, is_streak)) in cells.iter().enumerate() {
        let x = PAD + (cell_w + gap) * i as f32;
        let _ = write!(
            b.buf,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{cell_w:.1}" height="{cell_h:.1}" rx="8" fill="{}" stroke="{}" stroke-width="1"/>"#,
            theme.surface, theme.border
        );
        let val_color = if *is_streak {
            theme.text_streak
        } else {
            theme.text_primary
        };
        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="15" font-weight="500" font-family="monospace" fill="{}" text-anchor="middle">{}</text>"#,
            x + cell_w / 2.0,
            y + cell_h * 0.46,
            val_color,
            esc(val)
        );
        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="9" font-family="sans-serif" fill="{}" text-anchor="middle">{}</text>"#,
            x + cell_w / 2.0,
            y + cell_h * 0.80,
            theme.text_muted,
            esc(label)
        );
    }
    b.cursor_y += cell_h + 8.0;

    if matches!(variant, Variant::Stats) {
        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="9" font-family="sans-serif" fill="{}">Lifetime totals across all years since {}</text>"#,
            PAD,
            b.cursor_y + 4.0,
            theme.text_muted,
            stats.created_year
        );
        b.cursor_y += 12.0;
    }
}

fn render_highlight(b: &mut Builder, highlight: &str) {
    let theme = b.theme;
    let card_w = b.width;
    b.divider();
    b.cursor_y += 10.0;
    let y = b.cursor_y;
    let _ = write!(
        b.buf,
        r#"<rect x="{:.1}" y="{y:.1}" width="{:.1}" height="28" rx="6" fill="{}" stroke="{}" stroke-width="0.5"/>"#,
        PAD,
        card_w - PAD * 2.0,
        theme.surface,
        theme.accent_line
    );
    let _ = write!(
        b.buf,
        r#"<text x="{:.1}" y="{:.1}" font-size="11" font-family="sans-serif" fill="{}" text-anchor="middle">{}</text>"#,
        card_w / 2.0,
        y + 18.0,
        theme.text_accent,
        esc(highlight)
    );
    b.cursor_y += 28.0 + 8.0;
}

fn render_grid_and_langs(b: &mut Builder, stats: &GitHubStats, show_grid: bool, show_langs: bool) {
    b.divider();
    b.cursor_y += 4.0;
    let theme = b.theme;
    let start_y = b.cursor_y;
    let card_w = b.width;

    let cell_size = 9.0;
    let cell_gap = 2.0;
    let grid_w = if show_grid {
        18.0 * (cell_size + cell_gap) - cell_gap
    } else {
        0.0
    };
    let grid_h = if show_grid {
        7.0 * (cell_size + cell_gap) - cell_gap
    } else {
        0.0
    };

    if show_grid {
        b.section_label("CONTRIBUTION GRID · LAST 18 WEEKS", PAD, 16.0);
        let grid_top = start_y + 26.0;
        for (col, week) in stats.contribution_grid.iter().enumerate() {
            for (row, (_date, count)) in week.iter().enumerate() {
                let x = PAD + col as f32 * (cell_size + cell_gap);
                let y = grid_top + row as f32 * (cell_size + cell_gap);
                let _ = write!(
                    b.buf,
                    r#"<rect x="{x:.1}" y="{y:.1}" width="{cell_size}" height="{cell_size}" rx="2" fill="{}"/>"#,
                    theme.grid_color(*count)
                );
            }
        }
    }

    let lang_x = if show_grid {
        PAD + grid_w + 28.0
    } else {
        PAD
    };
    let lang_w = (card_w - PAD - lang_x).max(0.0);
    let langs_h: f32 = if show_langs {
        stats.top_languages.len() as f32 * 26.0
    } else {
        0.0
    };

    if show_langs {
        b.section_label("TOP LANGUAGES", lang_x, 16.0);
        let langs_top = start_y + 26.0;
        for (i, (name, color, pct)) in stats.top_languages.iter().enumerate() {
            let row_y = langs_top + i as f32 * 26.0;

            let _ = write!(
                b.buf,
                r#"<text x="{lang_x:.1}" y="{:.1}" font-size="11" font-family="sans-serif" fill="{}">{}</text>"#,
                row_y + 2.0,
                theme.text_primary,
                esc(name)
            );

            let _ = write!(
                b.buf,
                r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="monospace" fill="{}" text-anchor="end">{:.1}%</text>"#,
                lang_x + lang_w,
                row_y + 2.0,
                theme.text_muted,
                pct
            );

            let bar_y = row_y + 8.0;
            let _ = write!(
                b.buf,
                r#"<rect x="{lang_x:.1}" y="{bar_y:.1}" width="{lang_w:.1}" height="6" rx="3" fill="{}"/>"#,
                theme.surface
            );

            let fill_color = lang_color(color, i, theme);
            let fill_w = (lang_w * (*pct as f32 / 100.0)).max(0.0);
            let _ = write!(
                b.buf,
                r#"<rect x="{lang_x:.1}" y="{bar_y:.1}" width="{fill_w:.1}" height="6" rx="3" fill="{}"/>"#,
                fill_color
            );
        }
    }

    let consumed = grid_h.max(langs_h);
    b.cursor_y = start_y + 26.0 + consumed + 22.0;
}

fn render_top_repos(b: &mut Builder, stats: &GitHubStats, count: usize) {
    b.divider();
    b.cursor_y += 4.0;
    let theme = b.theme;
    let card_w = b.width;
    let start_y = b.cursor_y;

    b.section_label("PINNED REPOSITORIES", PAD, 16.0);

    let n = count.min(stats.pinned.len()).min(3);
    if n == 0 {
        b.cursor_y = start_y + 26.0 + 110.0 + 16.0;
        return;
    }

    let gap = 12.0_f32;
    let card_w_inner = (card_w - PAD * 2.0 - gap * (n as f32 - 1.0)) / n as f32;
    let card_h = 110.0_f32;
    let cards_top = start_y + 26.0;

    for (i, repo) in stats.pinned.iter().take(n).enumerate() {
        let x = PAD + (card_w_inner + gap) * i as f32;
        let _ = write!(
            b.buf,
            r#"<rect x="{x:.1}" y="{cards_top:.1}" width="{card_w_inner:.1}" height="{card_h:.1}" rx="10" fill="{}" stroke="{}" stroke-width="1"/>"#,
            theme.surface, theme.border
        );

        let dot_color = lang_color(&repo.lang_color, i, theme);
        let _ = write!(
            b.buf,
            r#"<rect x="{:.1}" y="{:.1}" width="8" height="8" rx="2" fill="{}"/>"#,
            x + 12.0,
            cards_top + 12.0,
            dot_color
        );

        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="12" font-weight="500" font-family="sans-serif" fill="{}">{}</text>"#,
            x + 26.0,
            cards_top + 20.0,
            theme.text_primary,
            esc(&repo.name)
        );

        let line_chars = ((card_w_inner - 20.0) / 6.2) as usize;
        let lines = wrap_two_lines(&repo.description, line_chars);
        for (li, line) in lines.iter().enumerate() {
            let _ = write!(
                b.buf,
                r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}">{}</text>"#,
                x + 12.0,
                cards_top + 44.0 + li as f32 * 14.0,
                theme.text_muted,
                esc(line)
            );
        }

        if !repo.language.is_empty() {
            let _ = write!(
                b.buf,
                r#"<rect x="{:.1}" y="{:.1}" width="8" height="8" rx="2" fill="{}"/>"#,
                x + 12.0,
                cards_top + card_h - 18.0,
                dot_color
            );
            let _ = write!(
                b.buf,
                r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}">{}</text>"#,
                x + 26.0,
                cards_top + card_h - 10.0,
                theme.text_muted,
                esc(&repo.language)
            );
        }

        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}" text-anchor="end">★ {}</text>"#,
            x + card_w_inner - 48.0,
            cards_top + card_h - 10.0,
            theme.text_primary,
            fmt_k(repo.stars)
        );
        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}" text-anchor="end">⑂ {}</text>"#,
            x + card_w_inner - 12.0,
            cards_top + card_h - 10.0,
            theme.text_muted,
            fmt_k(repo.forks)
        );
    }
    b.cursor_y = cards_top + card_h + 16.0;
}

fn render_contributed_to(b: &mut Builder, stats: &GitHubStats) {
    if stats.contributed_to.is_empty() {
        return;
    }
    b.divider();
    b.cursor_y += 4.0;
    let theme = b.theme;
    let card_w = b.width;
    let start_y = b.cursor_y;

    b.section_label("TOP CONTRIBUTED-TO REPOSITORIES", PAD, 16.0);

    let items: Vec<_> = stats.contributed_to.iter().take(3).collect();
    let n = items.len();
    let gap = 10.0_f32;
    let row_h = 36.0_f32;
    let item_w = (card_w - PAD * 2.0 - gap * (n as f32 - 1.0)) / n as f32;
    let row_y = start_y + 26.0;

    for (i, repo) in items.iter().enumerate() {
        let x = PAD + (item_w + gap) * i as f32;
        let _ = write!(
            b.buf,
            r#"<rect x="{x:.1}" y="{row_y:.1}" width="{item_w:.1}" height="{row_h:.1}" rx="6" fill="{}" stroke="{}" stroke-width="1"/>"#,
            theme.surface, theme.border
        );

        let dot_color = lang_color(&repo.lang_color, i, theme);
        let _ = write!(
            b.buf,
            r#"<rect x="{:.1}" y="{:.1}" width="8" height="8" rx="2" fill="{}"/>"#,
            x + 10.0,
            row_y + 14.0,
            dot_color
        );

        let name_max = ((item_w - 60.0) / 6.2) as usize;
        let name = truncate_chars(&repo.name_with_owner, name_max);
        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="11" font-family="sans-serif" fill="{}">{}</text>"#,
            x + 24.0,
            row_y + 22.0,
            theme.text_primary,
            esc(&name)
        );

        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="monospace" fill="{}" text-anchor="end">★ {}</text>"#,
            x + item_w - 10.0,
            row_y + 22.0,
            theme.text_muted,
            fmt_k(repo.stars)
        );
    }
    b.cursor_y = row_y + row_h + 14.0;
}

fn render_most_starred(b: &mut Builder, stats: &GitHubStats) {
    let Some(repo) = stats.most_starred.as_ref() else {
        return;
    };
    if repo.stars == 0 {
        return;
    }
    b.divider();
    b.cursor_y += 4.0;
    let theme = b.theme;
    let card_w = b.width;
    let start_y = b.cursor_y;

    b.section_label("MOST-STARRED REPOSITORY", PAD, 16.0);

    let tile_h = 56.0_f32;
    let tile_y = start_y + 26.0;
    let tile_w = card_w - PAD * 2.0;

    let _ = write!(
        b.buf,
        r#"<rect x="{:.1}" y="{tile_y:.1}" width="{tile_w:.1}" height="{tile_h:.1}" rx="8" fill="{}" stroke="{}" stroke-width="1"/>"#,
        PAD, theme.surface, theme.accent_line
    );

    let dot_color = lang_color(&repo.lang_color, 0, theme);
    let _ = write!(
        b.buf,
        r#"<rect x="{:.1}" y="{:.1}" width="10" height="10" rx="2" fill="{}"/>"#,
        PAD + 14.0,
        tile_y + 18.0,
        dot_color
    );

    let _ = write!(
        b.buf,
        r#"<text x="{:.1}" y="{:.1}" font-size="14" font-weight="500" font-family="sans-serif" fill="{}">{}</text>"#,
        PAD + 32.0,
        tile_y + 26.0,
        theme.text_primary,
        esc(&repo.name)
    );

    if !repo.language.is_empty() {
        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}">{}</text>"#,
            PAD + 32.0,
            tile_y + 44.0,
            theme.text_muted,
            esc(&repo.language)
        );
    }

    let _ = write!(
        b.buf,
        r#"<text x="{:.1}" y="{:.1}" font-size="14" font-weight="500" font-family="monospace" fill="{}" text-anchor="end">★ {}</text>"#,
        PAD + tile_w - 14.0,
        tile_y + 26.0,
        theme.text_streak,
        fmt_k(repo.stars)
    );
    let _ = write!(
        b.buf,
        r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}" text-anchor="end">⑂ {} forks</text>"#,
        PAD + tile_w - 14.0,
        tile_y + 44.0,
        theme.text_muted,
        fmt_k(repo.forks)
    );

    b.cursor_y = tile_y + tile_h + 16.0;
}

pub fn render(stats: &GitHubStats, theme: &Theme, config: &RenderConfig) -> String {
    let card_w = config.width;
    let sec = &config.sections;

    let mut total_h_f = 2.0_f32;
    if sec.header { total_h_f += 104.0; }
    if sec.stats {
        total_h_f += 8.0 + 52.0 + 14.0;
        if matches!(config.variant, Variant::Stats) {
            total_h_f += 12.0;
        }
    }
    if config.highlight.is_some() {
        total_h_f += 10.0 + 28.0 + 8.0;
    }
    if sec.grid || sec.languages {
        let grid_h: f32 = if sec.grid { 7.0 * 11.0 - 2.0 } else { 0.0 };
        let lang_h: f32 = if sec.languages {
            stats.top_languages.len() as f32 * 26.0
        } else {
            0.0
        };
        total_h_f += grid_h.max(lang_h) + 30.0;
    }
    if sec.top_repos && !stats.pinned.is_empty() {
        total_h_f += 110.0 + 40.0;
    }
    if sec.contributed_to && !stats.contributed_to.is_empty() {
        total_h_f += 36.0 + 40.0;
    }
    if sec.most_starred && stats.most_starred.as_ref().map(|r| r.stars > 0).unwrap_or(false) {
        total_h_f += 56.0 + 40.0;
    }
    total_h_f += 22.0; // footer
    total_h_f += 20.0; // buffer
    let total_h = total_h_f as u32;

    let mut b = Builder::new(theme, card_w);

    let login_esc = esc(&stats.login);
    let _ = write!(
        b.buf,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{card_w}" height="{total_h}" viewBox="0 0 {card_w} {total_h}" role="img"><title>GitHub stats for {login_esc}</title><desc>Profile card for {login_esc}</desc><defs><clipPath id="cc"><rect width="{card_w}" height="{total_h}" rx="16"/></clipPath></defs><g clip-path="url(#cc)"><rect width="{card_w}" height="{total_h}" fill="{}"/><rect width="{card_w}" height="2" fill="{}"/>"#,
        theme.bg, theme.accent_line
    );

    b.cursor_y = 10.0;

    if sec.header {
        render_header(&mut b, stats);
    }
    if sec.stats {
        render_stats_cells(&mut b, stats, config.variant);
    }
    if let Some(h) = &config.highlight {
        render_highlight(&mut b, h);
    }
    if sec.grid || sec.languages {
        render_grid_and_langs(&mut b, stats, sec.grid, sec.languages);
    }
    if sec.top_repos && !stats.pinned.is_empty() {
        render_top_repos(&mut b, stats, config.top_repos_count);
    }
    if sec.most_starred {
        render_most_starred(&mut b, stats);
    }
    if sec.contributed_to {
        render_contributed_to(&mut b, stats);
    }

    let now = chrono::Utc::now();
    let footer = format!(
        "Updated {:04}-{:02}-{:02} · statsvg.rs",
        now.year(),
        now.month(),
        now.day()
    );
    let _ = write!(
        b.buf,
        r##"<text font-size="9" font-family="sans-serif" fill="{}" x="{:.1}" y="{}" text-anchor="end">{}</text>"##,
        theme.text_muted,
        card_w - PAD,
        total_h.saturating_sub(8),
        esc(&footer)
    );
    if sec.border {
        let _ = write!(
            b.buf,
            r#"<rect width="{card_w}" height="{total_h}" rx="16" fill="none" stroke="{}" stroke-width="1"/>"#,
            theme.border
        );
    }
    b.buf.push_str("</g></svg>");

    b.buf
}
