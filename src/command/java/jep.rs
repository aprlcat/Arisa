use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    Context, Error,
    util::command::{check_cooldown, create_success_response},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JepMetadata {
    title: String,
    author: Option<String>,
    owner: Option<String>,
    jep_type: Option<String>,
    scope: Option<String>,
    status: Option<String>,
    release: Option<String>,
    component: Option<String>,
    discussion: Option<String>,
    reviewed_by: Option<String>,
    endorsed_by: Option<String>,
    created: Option<String>,
    updated: Option<String>,
    issue: Option<String>,
    summary: Option<String>,
    goals: Option<Vec<String>>,
    non_goals: Option<Vec<String>>,
    motivation: Option<String>,
    description: Option<String>,
}

#[derive(Clone)]
struct CachedJep {
    metadata: JepMetadata,
    cached_at: Instant,
}

lazy_static::lazy_static! {
    static ref JEP_CACHE: Arc<RwLock<HashMap<u16, CachedJep>>> = Arc::new(RwLock::new(HashMap::new()));
}

const CACHE_DURATION: Duration = Duration::from_secs(3600);

#[poise::command(
    slash_command,
    description_localized("en-US", "Get information about a Java Enhancement Proposal (JEP)")
)]
pub async fn jep(
    ctx: Context<'_>,
    #[description = "JEP number (e.g., 451)"] number: u16,
    #[description = "Show detailed sections (Goals, Non-Goals, etc.)"] detailed: Option<bool>,
) -> Result<(), Error> {
    check_cooldown(&ctx, "jep", ctx.data().config.cooldowns.github_cooldown).await?;
    ctx.defer().await?;

    let jep_info = match get_cached_jep(number).await {
        Some(cached) => cached,
        None => {
            let fetched = fetch_jep_info(number).await?;
            cache_jep(number, fetched.clone()).await;
            fetched
        }
    };

    let show_detailed = detailed.unwrap_or(false);

    let title = if jep_info.title.len() > 200 {
        format!("JEP {}: {}...", number, &jep_info.title[..180])
    } else {
        format!("JEP {}: {}", number, jep_info.title)
    };

    let description = format_jep_description(&jep_info, number, show_detailed);

    let final_description = if description.len() > 4000 {
        format!(
            "{}...\n\n*Output truncated. Use `/jep {} detailed:true` for full details or visit \
             the link below.*",
            &description[..3900],
            number
        )
    } else {
        description
    };

    let embed = create_success_response(&title, &final_description, false, &ctx.data().config);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn get_cached_jep(number: u16) -> Option<JepMetadata> {
    let cache = JEP_CACHE.read().await;
    if let Some(cached) = cache.get(&number) {
        if cached.cached_at.elapsed() < CACHE_DURATION {
            return Some(cached.metadata.clone());
        }
    }
    None
}

async fn cache_jep(number: u16, metadata: JepMetadata) {
    let mut cache = JEP_CACHE.write().await;
    cache.insert(
        number,
        CachedJep {
            metadata,
            cached_at: Instant::now(),
        },
    );

    cache.retain(|_, cached| cached.cached_at.elapsed() < CACHE_DURATION);
}

async fn fetch_jep_info(number: u16) -> Result<JepMetadata, Error> {
    let url = format!("https://openjdk.org/jeps/{}", number);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Arisa-Bot/1.0")
        .build()?;

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(crate::error::BotError::GitHub(format!(
            "JEP {} not found or inaccessible (HTTP {})",
            number,
            response.status()
        )));
    }

    let html = response.text().await?;
    parse_jep_html(&html, number)
}

fn parse_jep_html(html: &str, number: u16) -> Result<JepMetadata, Error> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    let title_selector = Selector::parse("h1").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .and_then(|element| {
            let text = element.text().collect::<Vec<_>>().join(" ");
            Some(text)
        })
        .unwrap_or_else(|| format!("JEP {}", number))
        .trim_start_matches(&format!("JEP {}: ", number))
        .to_string();

    let table_selector = Selector::parse("table.head tr").unwrap();
    let mut metadata = JepMetadata {
        title,
        author: None,
        owner: None,
        jep_type: None,
        scope: None,
        status: None,
        release: None,
        component: None,
        discussion: None,
        reviewed_by: None,
        endorsed_by: None,
        created: None,
        updated: None,
        issue: None,
        summary: None,
        goals: None,
        non_goals: None,
        motivation: None,
        description: None,
    };

    for row in document.select(&table_selector) {
        let cells: Vec<_> = row.select(&Selector::parse("td").unwrap()).collect();

        if cells.len() >= 2 {
            let key = clean_html(&cells[0].inner_html()).to_lowercase();
            let value = clean_html(&cells[1].inner_html());

            match key.as_str() {
                "author" => metadata.author = Some(value),
                "owner" => metadata.owner = Some(value),
                "type" => metadata.jep_type = Some(value),
                "scope" => metadata.scope = Some(value),
                "status" => metadata.status = Some(value),
                "release" => metadata.release = Some(value),
                "component" => metadata.component = Some(value),
                "discussion" => metadata.discussion = Some(value),
                "reviewed by" => metadata.reviewed_by = Some(value),
                "endorsed by" => metadata.endorsed_by = Some(value),
                "created" => metadata.created = Some(value),
                "updated" => metadata.updated = Some(value),
                "issue" => metadata.issue = Some(extract_issue_number(&cells[1].inner_html())),
                _ => {}
            }
        }
    }

    if let Some(summary_element) = document
        .select(&Selector::parse("#Summary + p").unwrap())
        .next()
    {
        let summary = clean_html(&summary_element.inner_html());
        if !summary.is_empty() {
            metadata.summary = Some(summary);
        }
    }

    let goals_selector = Selector::parse("#Goals + ul li").unwrap();
    let goals: Vec<String> = document
        .select(&goals_selector)
        .map(|element| clean_html(&element.inner_html()))
        .filter(|goal| !goal.is_empty())
        .collect();
    if !goals.is_empty() {
        metadata.goals = Some(goals);
    }

    let non_goals_selector = Selector::parse("#Non-Goals + ul li").unwrap();
    let non_goals: Vec<String> = document
        .select(&non_goals_selector)
        .map(|element| clean_html(&element.inner_html()))
        .filter(|goal| !goal.is_empty())
        .collect();
    if !non_goals.is_empty() {
        metadata.non_goals = Some(non_goals);
    }

    if let Some(motivation_element) = document
        .select(&Selector::parse("#Motivation + h3 + p").unwrap())
        .next()
    {
        let motivation = clean_html(&motivation_element.inner_html());
        if !motivation.is_empty() && motivation.len() < 800 {
            metadata.motivation = Some(motivation);
        }
    }

    if let Some(desc_element) = document
        .select(&Selector::parse("#Description + p").unwrap())
        .next()
    {
        let description = clean_html(&desc_element.inner_html());
        if !description.is_empty() && description.len() < 800 {
            metadata.description = Some(description);
        }
    }

    Ok(metadata)
}

fn clean_html(html: &str) -> String {
    use regex::Regex;

    let text = html
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#8201;", "")
        .replace("&#8201;/&#8201;", "/")
        .replace("&#8212;", "—")
        .replace("&#160;", " ")
        .replace("&nbsp;", " ");

    let tag_regex = Regex::new(r"<[^>]*>").unwrap();
    let cleaned = tag_regex.replace_all(&text, "");

    let whitespace_regex = Regex::new(r"\s+").unwrap();
    whitespace_regex
        .replace_all(&cleaned, " ")
        .trim()
        .to_string()
}

fn extract_issue_number(html: &str) -> String {
    use regex::Regex;

    let issue_regex = Regex::new(r"JDK-\d+").unwrap();
    if let Some(captures) = issue_regex.find(html) {
        return captures.as_str().to_string();
    }

    clean_html(html)
}

fn format_jep_description(jep: &JepMetadata, number: u16, detailed: bool) -> String {
    let mut description = String::new();

    let mut metadata_line = String::new();
    if let Some(status) = &jep.status {
        metadata_line.push_str(&format!("**Status:** {} ", status));
    }
    if let Some(release) = &jep.release {
        metadata_line.push_str(&format!("**Release:** {} ", release));
    }
    if let Some(jep_type) = &jep.jep_type {
        metadata_line.push_str(&format!("**Type:** {}", jep_type));
    }

    if !metadata_line.is_empty() {
        description.push_str(&metadata_line);
        description.push_str("\n\n");
    }

    if let Some(author) = &jep.author {
        description.push_str(&format!("**Author:** {}\n", author));
    }
    if let Some(owner) = &jep.owner {
        description.push_str(&format!("**Owner:** {}\n", owner));
    }

    if let Some(summary) = &jep.summary {
        description.push_str(&format!("\n**Summary**\n{}\n", summary));
    }

    if detailed {
        if let Some(goals) = &jep.goals {
            description.push_str("\n**Goals**\n");
            for (i, goal) in goals.iter().enumerate() {
                if i < 3 {
                    description.push_str(&format!("• {}\n", goal));
                }
            }
            if goals.len() > 3 {
                description.push_str(&format!("• ... and {} more goals\n", goals.len() - 3));
            }
        }

        if let Some(non_goals) = &jep.non_goals {
            description.push_str("\n**Non-Goals**\n");
            for (i, goal) in non_goals.iter().enumerate() {
                if i < 3 {
                    description.push_str(&format!("• {}\n", goal));
                }
            }
            if non_goals.len() > 3 {
                description.push_str(&format!(
                    "• ... and {} more non-goals\n",
                    non_goals.len() - 3
                ));
            }
        }

        if let Some(motivation) = &jep.motivation {
            description.push_str(&format!(
                "\n**Motivation**\n{}\n",
                if motivation.len() > 300 {
                    format!("{}...", &motivation[..300])
                } else {
                    motivation.clone()
                }
            ));
        }

        if let Some(desc) = &jep.description {
            description.push_str(&format!(
                "\n**Implementation**\n{}\n",
                if desc.len() > 300 {
                    format!("{}...", &desc[..300])
                } else {
                    desc.clone()
                }
            ));
        }

        if let Some(component) = &jep.component {
            description.push_str(&format!("\n**Component:** {}\n", component));
        }
        if let Some(reviewed_by) = &jep.reviewed_by {
            description.push_str(&format!("**Reviewed by:** {}\n", reviewed_by));
        }
        if let Some(endorsed_by) = &jep.endorsed_by {
            description.push_str(&format!("**Endorsed by:** {}\n", endorsed_by));
        }
    }

    if let Some(issue) = &jep.issue {
        description.push_str(&format!("\n**Issue:** {}\n", issue));
    }
    if let Some(created) = &jep.created {
        description.push_str(&format!("**Created:** {}\n", created));
    }

    description.push_str(&format!("\n**Link:** https://openjdk.org/jeps/{}", number));
    description
}
