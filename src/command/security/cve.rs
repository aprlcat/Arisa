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
struct CveData {
    id: String,
    #[serde(rename = "sourceIdentifier")]
    source_identifier: Option<String>,
    published: Option<String>,
    #[serde(rename = "lastModified")]
    last_modified: Option<String>,
    #[serde(rename = "vulnStatus")]
    vuln_status: Option<String>,
    descriptions: Vec<CveDescription>,
    metrics: Option<CveMetrics>,
    weaknesses: Option<Vec<CveWeakness>>,
    configurations: Option<Vec<CveConfiguration>>,
    references: Option<Vec<CveReference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CveDescription {
    lang: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CveMetrics {
    #[serde(rename = "cvssMetricV31")]
    cvss_v31: Option<Vec<CvssMetric>>,
    #[serde(rename = "cvssMetricV30")]
    cvss_v30: Option<Vec<CvssMetric>>,
    #[serde(rename = "cvssMetricV2")]
    cvss_v2: Option<Vec<CvssMetric>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CvssMetric {
    source: String,
    #[serde(rename = "type")]
    metric_type: String,
    #[serde(rename = "cvssData")]
    cvss_data: CvssData,
    #[serde(rename = "baseSeverity")]
    base_severity: Option<String>,
    #[serde(rename = "exploitabilityScore")]
    exploitability_score: Option<f64>,
    #[serde(rename = "impactScore")]
    impact_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CvssData {
    version: String,
    #[serde(rename = "baseScore")]
    base_score: f64,
    #[serde(rename = "baseSeverity")]
    base_severity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CveWeakness {
    source: String,
    #[serde(rename = "type")]
    weakness_type: String,
    description: Vec<CveDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CveConfiguration {
    nodes: Vec<CveNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CveNode {
    operator: Option<String>,
    #[serde(rename = "cpeMatch")]
    cpe_match: Option<Vec<CpeMatch>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpeMatch {
    vulnerable: bool,
    criteria: String,
    #[serde(rename = "versionStartIncluding")]
    version_start_including: Option<String>,
    #[serde(rename = "versionEndExcluding")]
    version_end_excluding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CveReference {
    url: String,
    source: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NvdResponse {
    #[serde(rename = "resultsPerPage")]
    results_per_page: u32,
    #[serde(rename = "startIndex")]
    start_index: u32,
    #[serde(rename = "totalResults")]
    total_results: u32,
    format: String,
    version: String,
    timestamp: String,
    vulnerabilities: Vec<CveVulnerability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CveVulnerability {
    cve: CveData,
}

#[derive(Clone)]
struct CachedCve {
    data: CveData,
    cached_at: Instant,
}

lazy_static::lazy_static! {
   static ref CVE_CACHE: Arc<RwLock<HashMap<String, CachedCve>>> = Arc::new(RwLock::new(HashMap::new()));
}

const CACHE_DURATION: Duration = Duration::from_secs(3600);

#[poise::command(
    slash_command,
    description_localized(
        "en-US",
        "Get information about a CVE (Common Vulnerabilities and Exposures)"
    )
)]
pub async fn cve(
    ctx: Context<'_>,
    #[description = "CVE ID (e.g., CVE-2019-16863)"] cve_id: String,
    #[description = "Show detailed technical information"] detailed: Option<bool>,
) -> Result<(), Error> {
    check_cooldown(&ctx, "cve", ctx.data().config.cooldowns.github_cooldown).await?;
    ctx.defer().await?;

    let normalized_id = normalize_cve_id(&cve_id)?;

    let cve_data = match get_cached_cve(&normalized_id).await {
        Some(cached) => cached,
        None => {
            let fetched = fetch_cve_info(&normalized_id).await?;
            cache_cve(&normalized_id, fetched.clone()).await;
            fetched
        }
    };

    let show_detailed = detailed.unwrap_or(false);
    let (title, description) = format_cve_response(&cve_data, show_detailed);

    let embed = create_success_response(&title, &description, false, &ctx.data().config);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn normalize_cve_id(input: &str) -> Result<String, Error> {
    let clean = input.trim().to_uppercase();

    if clean.starts_with("CVE-") {
        if clean.len() >= 13 && clean.chars().nth(8) == Some('-') {
            let year_part = &clean[4..8];
            let number_part = &clean[9..];

            if year_part.chars().all(|c| c.is_ascii_digit())
                && number_part.chars().all(|c| c.is_ascii_digit())
                && number_part.len() >= 4
            {
                return Ok(clean);
            }
        }
    } else if clean.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return Ok(format!("CVE-{}", clean));
    }

    Err(crate::error::BotError::InvalidFormat(
        "Invalid CVE format. Expected format: CVE-YYYY-NNNN (e.g., CVE-2019-16863)".to_string(),
    ))
}

async fn get_cached_cve(cve_id: &str) -> Option<CveData> {
    let cache = CVE_CACHE.read().await;
    if let Some(cached) = cache.get(cve_id) {
        if cached.cached_at.elapsed() < CACHE_DURATION {
            return Some(cached.data.clone());
        }
    }
    None
}

async fn cache_cve(cve_id: &str, data: CveData) {
    let mut cache = CVE_CACHE.write().await;
    cache.insert(
        cve_id.to_string(),
        CachedCve {
            data,
            cached_at: Instant::now(),
        },
    );

    cache.retain(|_, cached| cached.cached_at.elapsed() < CACHE_DURATION);
}

async fn fetch_cve_info(cve_id: &str) -> Result<CveData, Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Arisa-Bot/1.0")
        .build()?;

    let url = format!(
        "https://services.nvd.nist.gov/rest/json/cves/2.0?cveId={}",
        cve_id
    );

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(crate::error::BotError::GitHub(format!(
            "CVE {} not found or API error (HTTP {})",
            cve_id,
            response.status()
        )));
    }

    let nvd_response: NvdResponse = response.json().await?;

    if nvd_response.vulnerabilities.is_empty() {
        return Err(crate::error::BotError::GitHub(format!(
            "CVE {} not found in database",
            cve_id
        )));
    }

    Ok(nvd_response.vulnerabilities[0].cve.clone())
}

fn format_cve_response(cve: &CveData, detailed: bool) -> (String, String) {
    let mut description = String::new();

    if let Some(status) = &cve.vuln_status {
        description.push_str(&format!("**Status:** {}\n", status));
    }

    if let Some(published) = &cve.published {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(published) {
            description.push_str(&format!("**Published:** {}\n", dt.format("%Y-%m-%d")));
        }
    }

    if let Some(metrics) = &cve.metrics {
        if let Some(score_info) = get_best_cvss_score(metrics) {
            description.push_str(&format!(
                "**CVSS Score:** {} ({}) - {}\n",
                score_info.0, score_info.1, score_info.2
            ));
        }
    }

    if let Some(desc) = cve.descriptions.iter().find(|d| d.lang == "en") {
        let desc_text = if desc.value.len() > 500 {
            format!("{}...", &desc.value[..500])
        } else {
            desc.value.clone()
        };
        description.push_str(&format!("\n**Description:**\n{}\n", desc_text));
    }

    if detailed {
        if let Some(weaknesses) = &cve.weaknesses {
            if !weaknesses.is_empty() {
                description.push_str("\n**Weaknesses:**\n");
                for weakness in weaknesses.iter().take(3) {
                    if let Some(desc) = weakness.description.iter().find(|d| d.lang == "en") {
                        description.push_str(&format!("• {}\n", desc.value));
                    }
                }
            }
        }

        if let Some(configs) = &cve.configurations {
            description.push_str("\n**Affected Products:**\n");
            let mut product_count = 0;
            for config in configs.iter().take(2) {
                for node in &config.nodes {
                    if let Some(cpe_matches) = &node.cpe_match {
                        for cpe in cpe_matches.iter().take(3) {
                            if product_count >= 5 {
                                break;
                            }
                            let product = extract_product_from_cpe(&cpe.criteria);
                            description.push_str(&format!("• {}\n", product));
                            product_count += 1;
                        }
                    }
                    if product_count >= 5 {
                        break;
                    }
                }
                if product_count >= 5 {
                    break;
                }
            }
            if product_count >= 5 {
                description.push_str("• ... and more\n");
            }
        }
    }

    description.push_str(&format!(
        "\n**NVD Link:** https://nvd.nist.gov/vuln/detail/{}",
        cve.id
    ));

    if let Some(references) = &cve.references {
        if let Some(advisory) = references.iter().find(|r| {
            r.tags.as_ref().map_or(false, |tags| {
                tags.iter()
                    .any(|tag| tag.contains("Vendor") || tag.contains("Advisory"))
            })
        }) {
            description.push_str(&format!("\n**Vendor Advisory:** {}", advisory.url));
        }
    }

    (cve.id.clone(), description)
}

fn get_best_cvss_score(metrics: &CveMetrics) -> Option<(f64, String, String)> {
    if let Some(v31) = &metrics.cvss_v31 {
        if let Some(metric) = v31.first() {
            return Some((
                metric.cvss_data.base_score,
                format!("v{}", metric.cvss_data.version),
                metric
                    .base_severity
                    .clone()
                    .unwrap_or_else(|| severity_from_score(metric.cvss_data.base_score)),
            ));
        }
    }

    if let Some(v30) = &metrics.cvss_v30 {
        if let Some(metric) = v30.first() {
            return Some((
                metric.cvss_data.base_score,
                format!("v{}", metric.cvss_data.version),
                metric
                    .base_severity
                    .clone()
                    .unwrap_or_else(|| severity_from_score(metric.cvss_data.base_score)),
            ));
        }
    }

    if let Some(v2) = &metrics.cvss_v2 {
        if let Some(metric) = v2.first() {
            return Some((
                metric.cvss_data.base_score,
                format!("v{}", metric.cvss_data.version),
                severity_from_score(metric.cvss_data.base_score),
            ));
        }
    }

    None
}

fn severity_from_score(score: f64) -> String {
    match score {
        0.0 => "None".to_string(),
        0.1..=3.9 => "Low".to_string(),
        4.0..=6.9 => "Medium".to_string(),
        7.0..=8.9 => "High".to_string(),
        9.0..=10.0 => "Critical".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn extract_product_from_cpe(cpe: &str) -> String {
    let parts: Vec<&str> = cpe.split(':').collect();
    if parts.len() >= 5 {
        let vendor = parts.get(3).unwrap_or(&"unknown");
        let product = parts.get(4).unwrap_or(&"unknown");
        if *vendor != "*" && *product != "*" {
            format!("{} {}", vendor, product)
        } else if *product != "*" {
            product.to_string()
        } else {
            cpe.to_string()
        }
    } else {
        cpe.to_string()
    }
}
