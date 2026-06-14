# statsvg-rs

A small Rust web service that returns a styled SVG profile card for any GitHub user. Drop the URL into a README and it renders inline:

```markdown
![Stats](https://stats.yourdomain.com/api?username=akshay2211)
```

Five built-in themes, optional sections, no template engine, no SVG crate — the card is built as a `String` and gzipped on the way out.

## Quick start

```bash
cp .env.example .env
# edit .env — paste a GitHub classic PAT into GH_TOKEN

cargo run
curl "http://localhost:3000/api?username=akshay2211" -o card.svg
open card.svg          # macOS  (xdg-open on Linux)
```

Without `GH_TOKEN` the server still runs, but GitHub limits unauthenticated callers to 60 req/hr per IP, so `/api` will quickly return `502`.

## API

### `GET /api`

| Param             | Default       | Notes                                                       |
|-------------------|---------------|-------------------------------------------------------------|
| `username`        | *(required)*  | GitHub login                                                |
| `theme`           | `github_dark` | `github_dark` · `nord` · `dracula` · `light` · `solarized`  |
| `show_header`     | `true`        | accepts `true`/`false`/`0`/`1`/`yes`/`no`/`on`/`off`        |
| `show_stats`      | `true`        | "                                                           |
| `show_grid`       | `true`        | "                                                           |
| `show_languages`  | `true`        | "                                                           |
| `show_top_repos`  | `false`       | "                                                           |
| `top_repos_count` | `3`           | integer 1–6                                                 |

Returns `image/svg+xml; charset=utf-8` with `Cache-Control: public, max-age=1800, stale-while-revalidate=3600`.

### `GET /themes`

Returns the list of theme names as a JSON array.

### `GET /health`

Returns `200 ok` — wire this to your liveness probe.

## Embed examples

```markdown
<!-- Default -->
![Stats](https://stats.example.com/api?username=akshay2211)

<!-- Nord theme with top repositories -->
![Stats](https://stats.example.com/api?username=akshay2211&theme=nord&show_top_repos=true)

<!-- Light theme, repos only (no grid) -->
![Stats](https://stats.example.com/api?username=akshay2211&theme=light&show_grid=false&show_top_repos=true)

<!-- Minimal: just the 6 stat cells -->
![Stats](https://stats.example.com/api?username=akshay2211&show_header=false&show_grid=false&show_languages=false)
```

## Adding a theme

Edit `src/theme.rs`: declare a `pub const MY_THEME: Theme = Theme { ... }` and add `&MY_THEME` to `ALL_THEMES`. `/themes` and the `theme=` query param pick it up automatically — no other file changes needed.

## Docker

```bash
docker build -t statsvg-rs .
docker run -p 3000:3000 -e GH_TOKEN=ghp_xxx statsvg-rs
```

## Project layout

```
src/
  main.rs    — Axum router, app state, handlers
  github.rs  — GraphQL types, fetcher, stat computation
  svg.rs     — SVG string builder + section renderers
  theme.rs   — Theme struct and built-in theme constants
  params.rs  — query-string parsing + section toggle logic
  error.rs   — AppError enum + IntoResponse impl
```

The full implementation spec, including layout constants and per-section dimensions, lives in [`github-stats-rs-claude-code-prompt.md`](./github-stats-rs-claude-code-prompt.md).