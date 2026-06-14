# statsvg-rs

A Rust renderer that produces a styled SVG GitHub stats card and publishes it as a static file via GitHub Pages — no servers, no hosting bills, no API rate limits to manage. A scheduled GitHub Action re-renders the card every 6 hours so your README always shows fresh stats.

```markdown
![Stats](https://akshay2211.github.io/statsvg-rs/akshay2211.svg)
```

Five built-in themes, optional sections, no template engine, no SVG crate — the card is built as a `String` and gzipped on the way out.

## Setup (one time, for your own card)

1. **Fork this repo** and clone it.
2. **Edit `.github/workflows/render.yml`** — change `STATSVG_USER` to your GitHub login and optionally `STATSVG_THEME`:
   ```yaml
   env:
     STATSVG_USER: your-github-login
     STATSVG_THEME: github_dark    # or nord / dracula / light / solarized
   ```
3. **Enable GitHub Pages** in repo settings → Pages → Source = **GitHub Actions**.
4. **Push to main** — the workflow renders your card and publishes it. Your URL:
   ```
   https://<your-user>.github.io/<repo-name>/<your-user>.svg
   ```
5. **Embed it** in your profile README:
   ```markdown
   ![Stats](https://your-user.github.io/statsvg-rs/your-user.svg)
   ```

That's it. The card re-renders every 6 hours (cron in the workflow). You can also trigger it manually under **Actions → Render and publish stats SVG → Run workflow**.

No PATs or secrets to configure — the workflow uses Actions' auto-provided `GITHUB_TOKEN` for the GraphQL API call.

## Local development

```bash
cp .env.example .env
# Edit .env — paste a GitHub classic PAT into GH_TOKEN (only needed locally)

# Render a card to stdout
cargo run --release -- render --user akshay2211 --theme github_dark --show-top-repos > card.svg
open card.svg              # macOS  (xdg-open on Linux)

# Or run the HTTP server (useful for iterating on layout)
cargo run
curl "http://localhost:3000/api?username=akshay2211" -o card.svg
```

## Render CLI flags

| Flag                          | Default       | Notes                                                       |
|-------------------------------|---------------|-------------------------------------------------------------|
| `--user <login>`              | *(required)*  | GitHub login                                                |
| `--theme <name>`              | `github_dark` | `github_dark` · `nord` · `dracula` · `light` · `solarized`  |
| `--show-top-repos`            | off           | include the pinned-repos row                                |
| `--top-repos-count <1..6>`    | `3`           | how many pinned repos to show                               |
| `--no-header`                 | off           | hide profile header (avatar, name, bio)                     |
| `--no-stats`                  | off           | hide the 6-cell stats row                                   |
| `--no-grid`                   | off           | hide the contribution heatmap                               |
| `--no-languages`              | off           | hide the top-languages bars                                 |

## HTTP API (server mode, local only)

Run `cargo run` and the binary listens on `:3000`:

| Param             | Default       | Notes                                                       |
|-------------------|---------------|-------------------------------------------------------------|
| `username`        | *(required)*  | GitHub login                                                |
| `theme`           | `github_dark` | same set as `--theme`                                       |
| `show_header`     | `true`        | accepts `true`/`false`/`0`/`1`/`yes`/`no`/`on`/`off`        |
| `show_stats`      | `true`        | "                                                           |
| `show_grid`       | `true`        | "                                                           |
| `show_languages`  | `true`        | "                                                           |
| `show_top_repos`  | `false`       | "                                                           |
| `top_repos_count` | `3`           | integer 1–6                                                 |

Also: `GET /themes` (JSON list), `GET /health` (`200 ok`).

## Adding a theme

Edit `src/theme.rs`: declare a `pub const MY_THEME: Theme = Theme { ... }` and add `&MY_THEME` to `ALL_THEMES`. The `--theme` flag and `/themes` endpoint pick it up automatically.

## Project layout

```
src/
  main.rs    — CLI entry: `render` subcommand + HTTP server mode
  github.rs  — GraphQL types, fetcher, stat computation
  svg.rs     — SVG string builder + section renderers
  theme.rs   — Theme struct and built-in theme constants
  params.rs  — query-string parsing + section toggle logic
  error.rs   — AppError enum + IntoResponse impl
.github/workflows/
  render.yml — cron + push → renders SVG, deploys to GitHub Pages
```

The full implementation spec, including layout constants and per-section dimensions, lives in [`github-stats-rs-claude-code-prompt.md`](./github-stats-rs-claude-code-prompt.md).