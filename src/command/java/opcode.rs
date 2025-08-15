use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;
use poise::serenity_prelude::AutocompleteChoice;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    Context, Error,
    util::command::{check_cooldown, create_success_response},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JvmInstruction {
    #[serde(rename = "anchorId")]
    anchor_id: String,
    description: String,
    format: String,
    mnemonic: String,
    #[serde(rename = "operandStackAfter")]
    operand_stack_after: String,
    #[serde(rename = "operandStackBefore")]
    operand_stack_before: String,
    operation: String,
    opcode: Option<String>,
}

#[derive(Clone)]
struct CachedInstructions {
    instructions: HashMap<String, JvmInstruction>,
    cached_at: Instant,
}

static INSTRUCTION_CACHE: Lazy<Arc<RwLock<Option<CachedInstructions>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

static OPCODE_NAMES: Lazy<Vec<String>> = Lazy::new(|| {
    let instructions: Result<Vec<JvmInstruction>, _> = serde_json::from_str(JSON_DATA);

    match instructions {
        Ok(instructions) => {
            let mut names: Vec<String> = instructions
                .into_iter()
                .map(|instruction| instruction.mnemonic.to_lowercase())
                .collect();
            names.sort();
            names.dedup();
            names
        }
        Err(_) => Vec::new(),
    }
});

const CACHE_DURATION: Duration = Duration::from_secs(3600);
const JSON_DATA: &str = include_str!("../../../datagen/java/jvm_instructions.json");

async fn autocomplete_opcode(
    _ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = AutocompleteChoice> {
    let partial_lower = partial.to_lowercase();

    OPCODE_NAMES
        .iter()
        .filter(move |name| name.starts_with(&partial_lower))
        .take(25)
        .map(|name| AutocompleteChoice::new(name.clone(), name.clone()))
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Get information about a JVM bytecode instruction")
)]
pub async fn opcode(
    ctx: Context<'_>,
    #[description = "JVM instruction name (e.g., aaload, bipush, invokevirtual)"]
    #[autocomplete = "autocomplete_opcode"]
    instruction: String,
    #[description = "Show detailed stack information"] detailed: Option<bool>,
) -> Result<(), Error> {
    check_cooldown(
        &ctx,
        "opcode",
        ctx.data().config.cooldowns.per_user_cooldown,
    )
    .await?;

    let instructions = get_cached_instructions().await?;
    let instruction_key = instruction.trim().to_lowercase();

    let jvm_instruction = instructions.get(&instruction_key).ok_or_else(|| {
        crate::error::BotError::InvalidFormat(format!(
            "JVM instruction '{}' not found. Try checking the spelling or use a different \
             instruction name.",
            instruction
        ))
    })?;

    let show_detailed = detailed.unwrap_or(false);
    let description = format_instruction_info(jvm_instruction, show_detailed);

    let title = format!("JVM Instruction: {}", jvm_instruction.mnemonic);
    let embed = create_success_response(&title, &description, false, &ctx.data().config);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn get_cached_instructions() -> Result<HashMap<String, JvmInstruction>, Error> {
    let cache = INSTRUCTION_CACHE.read().await;

    if let Some(cached) = cache.as_ref() {
        if cached.cached_at.elapsed() < CACHE_DURATION {
            return Ok(cached.instructions.clone());
        }
    }

    drop(cache);

    let instructions = load_instructions()?;
    let mut cache = INSTRUCTION_CACHE.write().await;
    *cache = Some(CachedInstructions {
        instructions: instructions.clone(),
        cached_at: Instant::now(),
    });

    Ok(instructions)
}

fn load_instructions() -> Result<HashMap<String, JvmInstruction>, Error> {
    let instructions: Vec<JvmInstruction> =
        serde_json::from_str(JSON_DATA).map_err(|e| crate::error::BotError::Serialization(e))?;

    let mut map = HashMap::new();
    for instruction in instructions {
        map.insert(instruction.mnemonic.to_lowercase(), instruction.clone());

        if let Some(name) = instruction.anchor_id.strip_prefix("jvm-") {
            map.insert(name.to_lowercase(), instruction);
        }
    }

    Ok(map)
}

fn format_instruction_info(instruction: &JvmInstruction, detailed: bool) -> String {
    let mut description = String::new();

    if let Some(ref opcode_str) = instruction.opcode {
        description.push_str(&format!("**Opcode:** `{}`\n", opcode_str));
    }
    description.push_str(&format!("**Format:** `{}`\n\n", instruction.format));
    description.push_str(&format!("**Description:** {}\n", instruction.description));

    if detailed {
        description.push_str(&format!("**Operation:** {}\n", instruction.operation));

        description.push_str("\n**Stack Changes:**\n");
        description.push_str(&format!(
            "• Before: `{}`\n",
            instruction.operand_stack_before
        ));
        description.push_str(&format!("• After: `{}`\n", instruction.operand_stack_after));

        if let Some(ref opcode_str) = instruction.opcode {
            if let Some(opcode_info) = extract_opcode_number(opcode_str) {
                description.push_str(&format!("\n**Opcode Details:**\n"));
                description.push_str(&format!("• Decimal: {}\n", opcode_info.decimal));
                description.push_str(&format!("• Hexadecimal: {}\n", opcode_info.hex));
            }
        }
    }

    description.push_str(&format!(
        "\n[JVM Specification](https://docs.oracle.com/javase/specs/jvms/se24/html/jvms-6.html#{})",
        instruction.anchor_id
    ));

    description
}

struct OpcodeInfo {
    decimal: u8,
    hex: String,
}

fn extract_opcode_number(opcode_str: &str) -> Option<OpcodeInfo> {
    if let Some(parts) = opcode_str.split(" = ").nth(1) {
        if let Some(decimal_part) = parts.split(" (").next() {
            if let Ok(decimal) = decimal_part.parse::<u8>() {
                if let Some(hex_part) = parts.split("(").nth(1) {
                    if let Some(hex) = hex_part.strip_suffix(')') {
                        return Some(OpcodeInfo {
                            decimal,
                            hex: hex.to_string(),
                        });
                    }
                }
            }
        }
    }
    None
}
