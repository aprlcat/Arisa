use serde::Deserialize;

use crate::{Context, Error, util::command::{check_cooldown, create_success_response}};

#[derive(Deserialize)]
struct GitHubUser {
    login: String,
    name: Option<String>,
    bio: Option<String>,
    company: Option<String>,
    location: Option<String>,
    email: Option<String>,
    blog: Option<String>,
    public_repos: u32,
    public_gists: u32,
    followers: u32,
    following: u32,
    created_at: String,
    avatar_url: String,
}

#[derive(Deserialize)]
struct GitHubRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    language: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    watchers_count: u32,
    size: u32,
    created_at: String,
    updated_at: String,
    clone_url: String,
    html_url: String,
    #[serde(rename = "private")]
    is_private: bool,
    fork: bool,
    archived: bool,
    disabled: bool,
    license: Option<GitHubLicense>,
    topics: Vec<String>,
    default_branch: String,
    open_issues_count: u32,
}

#[derive(Deserialize)]
struct GitHubLicense {
    name: String,
    spdx_id: Option<String>,
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Get GitHub user or repository information")
)]
pub async fn github(
    ctx: Context<'_>,
    #[description = "GitHub username, repository, or URL"] input: String,
) -> Result<(), Error> {
    check_cooldown(&ctx, "github", ctx.data().config.cooldowns.github_cooldown).await?;

    let input = input.trim();
    let (user, repo) = parse_github_input(input);

    if let Some(repo_name) = repo {
        let (title, content) = get_repository_info(&user, &repo_name).await?;
        let embed = create_success_response(&title, &content, false, &ctx.data().config);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    } else {
        let (title, content, avatar_url) = get_user_info(&user).await?;
        let mut embed = create_success_response(&title, &content, false, &ctx.data().config);
        embed = embed.thumbnail(avatar_url);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }
    
    Ok(())
}

fn parse_github_input(input: &str) -> (String, Option<String>) {
    let input = input
        .strip_prefix("https://github.com/")
        .or_else(|| input.strip_prefix("http://github.com/"))
        .or_else(|| input.strip_prefix("github.com/"))
        .unwrap_or(input)
        .trim_end_matches('/');

    let parts: Vec<&str> = input.split('/').collect();

    match parts.len() {
        1 => (parts[0].to_string(), None),
        2 => (parts[0].to_string(), Some(parts[1].to_string())),
        _ => (parts[0].to_string(), None),
    }
}

async fn get_user_info(username: &str) -> Result<(String, String, String), Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/users/{}", username);

    let response = client
        .get(&url)
        .header("User-Agent", "Arisa-Bot/1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok((
            "User not found".to_string(),
            format!("Could not find GitHub user: {}", username),
            String::new(),
        ));
    }

    let user: GitHubUser = response.json().await?;

    let mut description = format!(
        "**Username:** {}\n**Public Repos:** {}\n**Followers:** {} | **Following:** {}\n**Public \
         Gists:** {}",
        user.login, user.public_repos, user.followers, user.following, user.public_gists
    );

    if let Some(name) = &user.name {
        description = format!("**Name:** {}\n{}", name, description);
    }

    if let Some(bio) = &user.bio {
        description.push_str(&format!("\n**Bio:** {}", bio));
    }

    if let Some(company) = &user.company {
        description.push_str(&format!("\n**Company:** {}", company));
    }

    if let Some(location) = &user.location {
        description.push_str(&format!("\n**Location:** {}", location));
    }

    if let Some(blog) = &user.blog {
        if !blog.is_empty() {
            description.push_str(&format!("\n**Website:** {}", blog));
        }
    }

    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&user.created_at) {
        description.push_str(&format!("\n**Joined:** {}", created.format("%B %d, %Y")));
    }

    description.push_str(&format!("\n**Profile:** https://github.com/{}", user.login));

    let title = format!("GitHub User: {}", user.login);
    Ok((title, description, user.avatar_url))
}

async fn get_repository_info(username: &str, repo_name: &str) -> Result<(String, String), Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}", username, repo_name);

    let response = client
        .get(&url)
        .header("User-Agent", "Arisa-Bot/1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok((
            "Repository not found".to_string(),
            format!("Could not find repository: {}/{}", username, repo_name),
        ));
    }

    let repo: GitHubRepo = response.json().await?;

    let mut description = format!(
        "**Owner:** {}\n**Stars:** {} | **Forks:** {} | **Watchers:** {}\n**Open Issues:** \
         {}\n**Size:** {} KB",
        username,
        repo.stargazers_count,
        repo.forks_count,
        repo.watchers_count,
        repo.open_issues_count,
        repo.size
    );

    if let Some(desc) = &repo.description {
        description = format!("**Description:** {}\n\n{}", desc, description);
    }

    if let Some(language) = &repo.language {
        description.push_str(&format!("\n**Language:** {}", language));
    }

    if let Some(license) = &repo.license {
        description.push_str(&format!("\n**License:** {}", license.name));
    }

    description.push_str(&format!("\n**Default Branch:** {}", repo.default_branch));

    if !repo.topics.is_empty() {
        description.push_str(&format!("\n**Topics:** {}", repo.topics.join(", ")));
    }

    let mut status_flags = Vec::new();
    if repo.is_private {
        status_flags.push("Private");
    }
    if repo.fork {
        status_flags.push("Fork");
    }
    if repo.archived {
        status_flags.push("Archived");
    }
    if repo.disabled {
        status_flags.push("Disabled");
    }

    if !status_flags.is_empty() {
        description.push_str(&format!("\n**Status:** {}", status_flags.join(" | ")));
    }

    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&repo.created_at) {
        description.push_str(&format!("\n**Created:** {}", created.format("%B %d, %Y")));
    }

    if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&repo.updated_at) {
        description.push_str(&format!(
            "\n**Last Updated:** {}",
            updated.format("%B %d, %Y")
        ));
    }

    description.push_str(&format!("\n**Repository:** {}", repo.html_url));

    let title = format!("Repository: {}", repo.full_name);
    Ok((title, description))
}