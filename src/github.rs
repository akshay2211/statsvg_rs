use std::collections::HashMap;

use serde::Deserialize;

use crate::error::AppError;

const STATS_QUERY: &str = r#"
query($login: String!) {
  user(login: $login) {
    name login bio company location avatarUrl
    followers { totalCount }
    following  { totalCount }
    repositories(first: 100, ownerAffiliations: OWNER, isFork: false, privacy: PUBLIC) {
      nodes {
        name description stargazerCount forkCount
        primaryLanguage { name color }
        languages(first: 10) {
          edges { size node { name color } }
        }
      }
    }
    contributionsCollection {
      totalCommitContributions
      totalPullRequestContributions
      totalIssueContributions
      totalPullRequestReviewContributions
      contributionCalendar {
        totalContributions
        weeks {
          contributionDays { contributionCount date }
        }
      }
    }
    pinnedItems(first: 6, types: [REPOSITORY]) {
      nodes {
        ... on Repository {
          name description stargazerCount forkCount
          primaryLanguage { name color }
        }
      }
    }
  }
}
"#;

#[derive(Debug, Deserialize)]
struct GqlResponse {
    data: Option<GqlData>,
}

#[derive(Debug, Deserialize)]
struct GqlData {
    user: Option<GqlUser>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlUser {
    name: Option<String>,
    login: String,
    bio: Option<String>,
    company: Option<String>,
    location: Option<String>,
    avatar_url: String,
    followers: Count,
    following: Count,
    repositories: RepoConnection,
    contributions_collection: ContributionsCollection,
    pinned_items: PinnedItems,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Count {
    total_count: u32,
}

#[derive(Debug, Deserialize)]
struct RepoConnection {
    nodes: Vec<RepoNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoNode {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    description: Option<String>,
    stargazer_count: u32,
    fork_count: u32,
    #[allow(dead_code)]
    primary_language: Option<Language>,
    languages: LanguageConnection,
}

#[derive(Debug, Deserialize)]
struct Language {
    name: String,
    color: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LanguageConnection {
    edges: Vec<LanguageEdge>,
}

#[derive(Debug, Deserialize)]
struct LanguageEdge {
    size: u64,
    node: Language,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContributionsCollection {
    total_commit_contributions: u32,
    total_pull_request_contributions: u32,
    total_issue_contributions: u32,
    total_pull_request_review_contributions: u32,
    contribution_calendar: ContributionCalendar,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContributionCalendar {
    total_contributions: u32,
    weeks: Vec<ContributionWeek>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContributionWeek {
    contribution_days: Vec<ContributionDay>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContributionDay {
    contribution_count: u32,
    date: String,
}

#[derive(Debug, Deserialize)]
struct PinnedItems {
    nodes: Vec<PinnedNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PinnedNode {
    name: Option<String>,
    description: Option<String>,
    stargazer_count: Option<u32>,
    fork_count: Option<u32>,
    primary_language: Option<Language>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GitHubStats {
    pub login: String,
    pub name: String,
    pub bio: String,
    pub company: String,
    pub location: String,
    pub avatar_url: String,
    pub followers: u32,
    pub following: u32,

    pub total_contributions: u32,
    pub total_commits: u32,
    pub total_prs: u32,
    pub total_issues: u32,
    pub total_reviews: u32,

    pub current_streak: u32,
    pub longest_streak: u32,

    pub total_stars: u32,
    pub total_forks: u32,
    pub public_repos: u32,

    pub top_languages: Vec<(String, String, f64)>,

    pub contribution_grid: Vec<Vec<(String, u32)>>,

    pub pinned: Vec<PinnedRepo>,
}

#[derive(Debug, Clone)]
pub struct PinnedRepo {
    pub name: String,
    pub description: String,
    pub stars: u32,
    pub forks: u32,
    pub language: String,
    pub lang_color: String,
}

pub async fn fetch_stats(
    client: &reqwest::Client,
    token: &str,
    login: &str,
) -> Result<GitHubStats, AppError> {
    let body = serde_json::json!({
        "query": STATS_QUERY,
        "variables": { "login": login },
    });

    let mut req = client
        .post("https://api.github.com/graphql")
        .header("User-Agent", "statsvg-rs/0.1")
        .json(&body);
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let resp: GqlResponse = req.send().await?.error_for_status()?.json().await?;

    let user = resp
        .data
        .and_then(|d| d.user)
        .ok_or_else(|| AppError::NotFound(login.to_string()))?;

    let repos = &user.repositories.nodes;
    let total_stars: u32 = repos.iter().map(|r| r.stargazer_count).sum();
    let total_forks: u32 = repos.iter().map(|r| r.fork_count).sum();
    let public_repos = repos.len() as u32;

    let top_languages = compute_languages(repos);
    let calendar = &user.contributions_collection.contribution_calendar;
    let (current_streak, longest_streak) = compute_streaks(calendar);
    let contribution_grid = build_grid(calendar);

    let login_owned = user.login.clone();
    let pinned = user
        .pinned_items
        .nodes
        .into_iter()
        .filter_map(|p| {
            Some(PinnedRepo {
                name: p.name?,
                description: p.description.unwrap_or_default(),
                stars: p.stargazer_count.unwrap_or(0),
                forks: p.fork_count.unwrap_or(0),
                language: p
                    .primary_language
                    .as_ref()
                    .map(|l| l.name.clone())
                    .unwrap_or_default(),
                lang_color: p
                    .primary_language
                    .and_then(|l| l.color)
                    .unwrap_or_else(|| "#888".into()),
            })
        })
        .collect();

    Ok(GitHubStats {
        name: user.name.unwrap_or_else(|| login_owned.clone()),
        login: login_owned,
        bio: user.bio.unwrap_or_default(),
        company: user.company.unwrap_or_default(),
        location: user.location.unwrap_or_default(),
        avatar_url: user.avatar_url,
        followers: user.followers.total_count,
        following: user.following.total_count,
        total_contributions: calendar.total_contributions,
        total_commits: user.contributions_collection.total_commit_contributions,
        total_prs: user.contributions_collection.total_pull_request_contributions,
        total_issues: user.contributions_collection.total_issue_contributions,
        total_reviews: user
            .contributions_collection
            .total_pull_request_review_contributions,
        current_streak,
        longest_streak,
        total_stars,
        total_forks,
        public_repos,
        top_languages,
        contribution_grid,
        pinned,
    })
}

fn compute_languages(repos: &[RepoNode]) -> Vec<(String, String, f64)> {
    let mut totals: HashMap<String, (u64, String)> = HashMap::new();
    for repo in repos {
        for edge in &repo.languages.edges {
            let color = edge
                .node
                .color
                .clone()
                .unwrap_or_else(|| "#888".into());
            totals
                .entry(edge.node.name.clone())
                .and_modify(|(bytes, _)| *bytes += edge.size)
                .or_insert((edge.size, color));
        }
    }
    let total_bytes: u64 = totals.values().map(|(b, _)| *b).sum();
    if total_bytes == 0 {
        return Vec::new();
    }
    let mut langs: Vec<(String, String, f64)> = totals
        .into_iter()
        .map(|(name, (bytes, color))| (name, color, (bytes as f64 / total_bytes as f64) * 100.0))
        .collect();
    langs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    langs.truncate(5);
    langs
}

fn compute_streaks(calendar: &ContributionCalendar) -> (u32, u32) {
    let counts: Vec<u32> = calendar
        .weeks
        .iter()
        .flat_map(|w| w.contribution_days.iter().map(|d| d.contribution_count))
        .collect();

    let mut current = 0u32;
    for &c in counts.iter().rev() {
        if c > 0 {
            current += 1;
        } else {
            break;
        }
    }

    let mut longest = 0u32;
    let mut run = 0u32;
    for &c in &counts {
        if c > 0 {
            run += 1;
            if run > longest {
                longest = run;
            }
        } else {
            run = 0;
        }
    }

    (current, longest)
}

fn build_grid(calendar: &ContributionCalendar) -> Vec<Vec<(String, u32)>> {
    let weeks = &calendar.weeks;
    let start = weeks.len().saturating_sub(18);
    weeks[start..]
        .iter()
        .map(|w| {
            w.contribution_days
                .iter()
                .map(|d| (d.date.clone(), d.contribution_count))
                .collect()
        })
        .collect()
}