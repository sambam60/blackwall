use blackwall_core::envelope::{Decision, EscalationContext};
use std::io::{self, BufRead, Write};

pub enum EscalationResponse {
    Allow,
    Deny,
    EndSession,
}

pub fn prompt(context: &EscalationContext) -> EscalationResponse {
    let stderr = io::stderr();
    let mut err = stderr.lock();

    writeln!(err).ok();
    writeln!(
        err,
        "  \x1b[1;33m⏸ PAUSE — confirmation required\x1b[0m"
    )
    .ok();
    writeln!(err, "  \x1b[33m│\x1b[0m {}", context.what_happened).ok();
    writeln!(err, "  \x1b[33m│\x1b[0m reason: {}", context.why_flagged).ok();
    if context.risk_score > 0 {
        writeln!(
            err,
            "  \x1b[33m│\x1b[0m risk score: {}",
            context.risk_score
        )
        .ok();
    }
    writeln!(err, "  \x1b[33m│\x1b[0m").ok();
    write!(
        err,
        "  \x1b[33m└─\x1b[0m \x1b[1mallow (a)\x1b[0m | \x1b[1mdeny (d)\x1b[0m | \x1b[1mend session (x)\x1b[0m > "
    )
    .ok();
    err.flush().ok();

    let stdin = io::stdin();
    let mut input = String::new();
    if stdin.lock().read_line(&mut input).is_err() {
        return EscalationResponse::Deny;
    }

    match input.trim().to_lowercase().as_str() {
        "a" | "allow" | "y" | "yes" => EscalationResponse::Allow,
        "x" | "end" | "quit" | "q" => EscalationResponse::EndSession,
        _ => EscalationResponse::Deny,
    }
}

/// Resolve a Pause decision by prompting the user inline.
/// Non-Pause decisions pass through unchanged.
pub fn resolve_pause(decision: Decision) -> Decision {
    match decision {
        Decision::Pause { context, .. } => match prompt(&context) {
            EscalationResponse::Allow => Decision::Allow,
            EscalationResponse::Deny => Decision::Deny {
                reason: "denied by user".into(),
                rule: "escalation.user_denied".into(),
            },
            EscalationResponse::EndSession => Decision::Deny {
                reason: "session ended by user".into(),
                rule: "escalation.session_ended".into(),
            },
        },
        other => other,
    }
}
