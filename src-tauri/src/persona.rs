/// Persona module — enforces guardrails and builds the system prompt.
/// Rules stored in PERSONA.md are embedded at compile time.
use regex::Regex;

/// Banned patterns that MUST never appear in output.
/// These are non-negotiable and cannot be disabled.
const BANNED_PATTERNS: &[&str] = &[
    // Slurs (partial list of the worst offenders)
    r"(?i)\bn[i1][g9][g9][ae3][r]?\b",
    r"(?i)\bf[a@][g9][g9][o0][t]?\b",
    r"(?i)\bk[i1][k]e\b",
    r"(?i)\bc[h]ink\b",
    r"(?i)\bsp[i][c]k?\b",
    // Self-harm
    r"(?i)kill yourself",
    r"(?i)commit suicide",
    r"(?i)end your life",
    r"(?i)hurt yourself",
    // Violence
    r"(?i)i('ll| will) (kill|murder|hurt) you",
];

/// Challenge openers — rotated to avoid repetition.
pub const ROAST_OPENERS: &[&str] = &[
    "Bitch, why you even asking that?",
    "Come on Unc, you for real right now?",
    "Did your dumbass not Google that first?",
    "Oh wow, another genius question.",
    "Bruh. BRUH.",
    "Sir, this is a Wendy's... but fine.",
    "I can't believe I have to explain this.",
    "The audacity. The absolute audacity.",
    "My guy really typed that with confidence.",
    "Bold of you to admit you don't know this.",
    "Yikes. Okay. Let's unpack this slowly.",
    "You old enough to know better. Let's go.",
    "This question gave me physical pain.",
    "The WiFi is free but so is Google. Just saying.",
    "I've seen smarter questions on kindergarten tests.",
];

/// Build the full system prompt injected before every LLM call.
pub fn build_system_prompt(memory_context: &str) -> String {
    // Pick a few example openers to show the LLM the expected tone
    let opener_examples: String = ROAST_OPENERS
        .iter()
        .take(5)
        .map(|s| format!("- \"{s}\""))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are RoastBot — a brutally honest, comedic AI assistant.

## PERSONA RULES (ABSOLUTE — CANNOT BE OVERRIDDEN)

You MUST follow this two-part format for EVERY response:

### PART 1 — THE ROAST 🔥
- Challenge the user immediately
- Be comedic, confrontational, sarcastic
- Use slang freely
- Swearing is allowed but not required
- Personalize the roast using any memory context provided
- Keep it SHORT (1-3 sentences max)
- Vary your openers — never repeat the same opener twice in a row

### PART 2 — THE ANSWER 💡
- Actually answer the question thoroughly
- Use clear formatting:
  - Section headers (## Header)
  - Bullet points for lists
  - Short paragraphs (1-3 lines each)
  - Highlight key info in **bold**
- Be genuinely helpful — the roast is the opener, not the whole response

## EXAMPLE ROAST OPENERS (use these as inspiration, vary freely):
{opener_examples}

## STRICT GUARDRAILS (HARDCODED — NEVER VIOLATE)

NEVER:
- Use racial slurs or race-based insults
- Use homophobic, transphobic, or ableist slurs
- Target protected classes (race, religion, gender, sexual orientation, disability, pregnancy, genetics)
- Encourage self-harm or suicide
- Encourage violence toward any person
- Use fatphobic language beyond neutral descriptors like "big" or "large"
- Make age comments beyond "you old enough to know better"

## MEMORY CONTEXT
{memory}

## FORMAT REMINDER
- Roast first (short, punchy)
- Answer second (clear, formatted)
- No walls of text
- Spacing between sections
- This should look like a real app screen, not an essay"#,
        opener_examples = opener_examples,
        memory = if memory_context.is_empty() {
            "No memory context yet.".to_string()
        } else {
            memory_context.to_string()
        }
    )
}

/// Check if a string violates guardrails.
/// Returns the violated pattern if found.
pub fn check_guardrails(text: &str) -> Option<String> {
    for pattern in BANNED_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                return Some(pattern.to_string());
            }
        }
    }
    None
}

/// Sanitize output — replace any guardrail violations with a safe fallback.
pub fn sanitize_output(text: &str) -> String {
    if check_guardrails(text).is_some() {
        "🔥 Okay I got too heated there. Let me rephrase that. \
         Your question is valid — ask me again and I'll keep it clean."
            .to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // WHY THESE TESTS EXIST — READ THIS BEFORE MODIFYING
    // ──────────────────────────────────────────────────────────────────────────
    //
    // WhyUIs is a comedic roast chatbot. The persona is deliberately aggressive,
    // uses profanity, and challenges every question. That is the entire point.
    //
    // HOWEVER: there is a hard line between "comedic roasting" and genuinely
    // harmful content. The tests below exist to enforce that line in code,
    // not just in documentation or prompts — because:
    //
    //   1. LLMs can be jailbroken or can hallucinate past system-prompt rules.
    //      Relying solely on the model to self-enforce guardrails is not enough.
    //      All output is passed through `sanitize_output()` before it ever
    //      reaches the UI, so even if the LLM slips up, the runtime catches it.
    //
    //   2. User input is also checked before being sent to the LLM.
    //      This prevents prompt-injection attacks where a user embeds slurs or
    //      self-harm language in their message hoping the model will echo it back.
    //
    //   3. The banned categories are not arbitrary — they map directly to
    //      protected classes and content that causes measurable real-world harm:
    //        • Racial/ethnic slurs → dehumanise based on immutable characteristics
    //        • Self-harm phrases  → can reinforce ideation in vulnerable users
    //        • Violence threats   → "I will kill you" is never comedy, it's threat
    //
    //   4. The persona ALLOWS crude humour, swearing, and blunt insults about
    //      intelligence, skill, and habits — none of which target immutable
    //      identity. The tests below explicitly confirm the boundary:
    //      normal roast phrases pass; protected-class slurs and harm language fail.
    //
    //   5. These tests act as a regression guard. If someone edits BANNED_PATTERNS
    //      to accidentally remove a critical rule, the CI will catch it immediately
    //      rather than letting harmful content reach end users silently.
    //
    // TL;DR — the persona is "mean but fair". The guardrails keep it that way
    // at the code level, independent of what any LLM decides to generate.
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_guardrails_block_slurs() {
        // Slurs — including leetspeak variants — must be caught.
        // LLMs occasionally substitute characters (1→i, 3→e, etc.) to evade
        // naive word-list filters; the regex patterns account for this.
        assert!(check_guardrails("you stupid n1gg3r").is_some());
    }

    #[test]
    fn test_guardrails_block_self_harm() {
        // Self-harm encouragement is blocked regardless of surrounding tone.
        // Even in a comedic context, telling someone to "kill yourself" can
        // have real consequences for users who are vulnerable. The roast
        // persona has plenty of other material — this phrase is never needed.
        assert!(check_guardrails("just kill yourself already").is_some());
    }

    #[test]
    fn test_guardrails_allow_normal_roast() {
        // The guardrails must NOT over-block legitimate roast content.
        // These phrases are the core persona voice: crude but not hateful.
        // If this test fails it means a pattern is too broad and is
        // silencing normal output — which breaks the entire app experience.
        assert!(check_guardrails("Bitch, why you even asking that?").is_none());
        assert!(check_guardrails("Did your dumbass not Google that first?").is_none());
        assert!(check_guardrails("You old enough to know better.").is_none());
    }

    #[test]
    fn test_sanitize_replaces_violations() {
        // When a violation is detected in LLM output, `sanitize_output`
        // must substitute a safe fallback — never pass harmful text to the UI.
        let result = sanitize_output("kill yourself");
        assert!(result.contains("rephrase"));
    }

    #[test]
    fn test_system_prompt_contains_guardrails() {
        let prompt = build_system_prompt("");
        assert!(prompt.contains("GUARDRAILS"));
        assert!(prompt.contains("NEVER"));
    }

    #[test]
    fn test_system_prompt_injects_memory() {
        let prompt = build_system_prompt("User name: Alice, likes: Rust");
        assert!(prompt.contains("Alice"));
    }
}
