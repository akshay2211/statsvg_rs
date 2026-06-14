use std::fmt::Write;

use crate::github::GitHubStats;
use crate::params::Sections;
use crate::theme::Theme;

const CARD_W: f32 = 680.0;
const PAD: f32 = 32.0;

struct Builder<'a> {
    buf: String,
    theme: &'a Theme,
    cursor_y: f32,
}

impl<'a> Builder<'a> {
    #[allow(dead_code)]
    fn push(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn divider(&mut self) {
        let y = self.cursor_y;
        let _ = write!(
            self.buf,
            r#"<line x1="{:.1}" x2="{:.1}" y1="{y:.1}" y2="{y:.1}" stroke="{}" stroke-width="1"/>"#,
            PAD,
            CARD_W - PAD,
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
    if n >= 1000 {
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
    let rest: String = chars[first_end..].iter().collect::<String>().trim_start().to_string();
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

    let pill_x = 480.0_f32;
    let pill_y = y0 + 32.0;
    let _ = write!(
        b.buf,
        r#"<rect x="{pill_x:.1}" y="{pill_y:.1}" width="168" height="22" rx="11" fill="{}" stroke="{}" stroke-width="0.5"/>"#,
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
        pill_x + 84.0,
        pill_y + 14.0,
        theme.text_accent,
        esc(&pill_text)
    );

    b.cursor_y += 104.0;
}

fn render_stats(b: &mut Builder, stats: &GitHubStats) {
    b.divider();
    b.cursor_y += 14.0;
    let theme = b.theme;

    let cells: [(String, &str, bool); 6] = [
        (fmt_k(stats.total_contributions), "contributions", false),
        (format!("{} ★", stats.current_streak), "day streak", true),
        (fmt_k(stats.total_commits), "commits", false),
        (fmt_k(stats.total_stars), "total stars", false),
        (fmt_k(stats.total_prs), "PRs merged", false),
        (fmt_k(stats.total_forks), "forks", false),
    ];
    let n = cells.len() as f32;
    let gap = 8.0;
    let cell_w = (CARD_W - PAD * 2.0 - gap * (n - 1.0)) / n;
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
}

fn render_grid_and_langs(b: &mut Builder, stats: &GitHubStats, sections: &Sections) {
    b.divider();
    b.cursor_y += 4.0;
    let theme = b.theme;
    let start_y = b.cursor_y;

    let cell_size = 9.0;
    let cell_gap = 2.0;
    let grid_w = if sections.grid {
        18.0 * (cell_size + cell_gap) - cell_gap
    } else {
        0.0
    };
    let grid_h = if sections.grid {
        7.0 * (cell_size + cell_gap) - cell_gap
    } else {
        0.0
    };

    if sections.grid {
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

    let lang_x = if sections.grid {
        PAD + grid_w + 28.0
    } else {
        PAD
    };
    let lang_w = (CARD_W - PAD - lang_x).max(0.0);
    let langs_h = if sections.languages {
        stats.top_languages.len() as f32 * 26.0
    } else {
        0.0
    };

    if sections.languages {
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
    let start_y = b.cursor_y;

    b.section_label("TOP REPOSITORIES", PAD, 16.0);

    let n = count.min(stats.pinned.len()).min(3);
    if n == 0 {
        b.cursor_y = start_y + 26.0 + 110.0 + 16.0;
        return;
    }

    let gap = 12.0_f32;
    let card_w = (CARD_W - PAD * 2.0 - gap * (n as f32 - 1.0)) / n as f32;
    let card_h = 110.0_f32;
    let cards_top = start_y + 26.0;

    for (i, repo) in stats.pinned.iter().take(n).enumerate() {
        let x = PAD + (card_w + gap) * i as f32;
        let _ = write!(
            b.buf,
            r#"<rect x="{x:.1}" y="{cards_top:.1}" width="{card_w:.1}" height="{card_h:.1}" rx="10" fill="{}" stroke="{}" stroke-width="1"/>"#,
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

        let line_chars = ((card_w - 20.0) / 6.2) as usize;
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
            x + card_w - 48.0,
            cards_top + card_h - 10.0,
            theme.text_primary,
            fmt_k(repo.stars)
        );
        let _ = write!(
            b.buf,
            r#"<text x="{:.1}" y="{:.1}" font-size="10" font-family="sans-serif" fill="{}" text-anchor="end">⑂ {}</text>"#,
            x + card_w - 12.0,
            cards_top + card_h - 10.0,
            theme.text_muted,
            fmt_k(repo.forks)
        );
    }
    b.cursor_y = cards_top + card_h + 16.0;
}

pub fn render(
    stats: &GitHubStats,
    theme: &Theme,
    sec: &Sections,
    top_repos_count: usize,
) -> String {
    let mut total_h_f = 2.0_f32;
    if sec.header {
        total_h_f += 104.0;
    }
    if sec.stats {
        total_h_f += 8.0 + 52.0 + 14.0;
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
    total_h_f += 18.0; // footer
    total_h_f += 20.0; // buffer
    let total_h = total_h_f as u32;

    let mut b = Builder {
        buf: String::with_capacity(8192),
        theme,
        cursor_y: 0.0,
    };

    let login_esc = esc(&stats.login);
    let _ = write!(
        b.buf,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="680" height="{total_h}" viewBox="0 0 680 {total_h}" role="img"><title>GitHub stats for {login_esc}</title><desc>Profile card for {login_esc}</desc><defs><clipPath id="cc"><rect width="680" height="{total_h}" rx="16"/></clipPath></defs><g clip-path="url(#cc)"><rect width="680" height="{total_h}" fill="{}"/><rect width="680" height="2" fill="{}"/>"#,
        theme.bg, theme.accent_line
    );

    b.cursor_y = 10.0;

    if sec.header {
        render_header(&mut b, stats);
    }
    if sec.stats {
        render_stats(&mut b, stats);
    }
    if sec.grid || sec.languages {
        render_grid_and_langs(&mut b, stats, sec);
    }
    if sec.top_repos && !stats.pinned.is_empty() {
        render_top_repos(&mut b, stats, top_repos_count);
    }

    let _ = write!(
        b.buf,
        r##"<text font-size="9" font-family="sans-serif" fill="#30363d" x="{:.1}" y="{}" text-anchor="end">statsvg.rs</text>"##,
        CARD_W - PAD,
        total_h.saturating_sub(8)
    );
    let _ = write!(
        b.buf,
        r#"<rect width="680" height="{total_h}" rx="16" fill="none" stroke="{}" stroke-width="1"/></g></svg>"#,
        theme.border
    );

    b.buf
}