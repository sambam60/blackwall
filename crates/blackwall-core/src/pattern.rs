use crate::envelope::{Action, Decision, EscalationContext};
use crate::policy::{CircuitAction, Pattern, PatternStep};
use std::collections::VecDeque;

struct RecordedAction {
    action_key: String,
    target: String,
    _sequence: u64,
}

struct CompiledPattern {
    name: String,
    description: String,
    steps: Vec<PatternStep>,
    within: usize,
    risk: u32,
    on_match: CircuitAction,
}

pub struct PatternMatcher {
    patterns: Vec<CompiledPattern>,
    history: VecDeque<RecordedAction>,
    max_window: usize,
}

impl PatternMatcher {
    pub fn new(patterns: &[Pattern]) -> Self {
        let max_window = patterns.iter().map(|p| p.within).max().unwrap_or(50);

        let compiled = patterns
            .iter()
            .map(|p| CompiledPattern {
                name: p.name.clone(),
                description: p.description.clone(),
                steps: p.sequence.clone(),
                within: p.within,
                risk: p.risk,
                on_match: p.on_match.clone(),
            })
            .collect();

        Self {
            patterns: compiled,
            history: VecDeque::with_capacity(max_window),
            max_window,
        }
    }

    pub fn record_and_check(&mut self, action: &Action, sequence: u64) -> Option<Decision> {
        self.history.push_back(RecordedAction {
            action_key: format!("{}.{}", action.tool, action.operation),
            target: action.target.clone(),
            _sequence: sequence,
        });

        while self.history.len() > self.max_window {
            self.history.pop_front();
        }

        for pattern in &self.patterns {
            if self.check_pattern(pattern) {
                let decision = match pattern.on_match {
                    CircuitAction::Halt => Decision::Deny {
                        reason: format!("pattern '{}' matched: {}", pattern.name, pattern.description),
                        rule: format!("patterns.{}", pattern.name),
                    },
                    CircuitAction::Pause | CircuitAction::Degrade | CircuitAction::Isolate => {
                        Decision::Pause {
                            reason: format!(
                                "pattern '{}' detected: {}",
                                pattern.name, pattern.description
                            ),
                            context: EscalationContext {
                                what_happened: format!(
                                    "action sequence matched anti-pattern '{}'",
                                    pattern.name
                                ),
                                why_flagged: pattern.description.clone(),
                                risk_score: pattern.risk,
                            },
                        }
                    }
                };
                return Some(decision);
            }
        }

        None
    }

    fn check_pattern(&self, pattern: &CompiledPattern) -> bool {
        if pattern.steps.is_empty() {
            return false;
        }

        let window_start = if self.history.len() > pattern.within {
            self.history.len() - pattern.within
        } else {
            0
        };

        let mut step_idx = 0;
        for recorded in self.history.iter().skip(window_start) {
            if step_idx >= pattern.steps.len() {
                break;
            }
            if step_matches(&pattern.steps[step_idx], recorded) {
                step_idx += 1;
            }
        }

        step_idx >= pattern.steps.len()
    }
}

fn step_matches(step: &PatternStep, recorded: &RecordedAction) -> bool {
    if recorded.action_key != step.action {
        return false;
    }

    if let Some(ref path_glob) = step.path {
        if !simple_wildcard_match(path_glob, &recorded.target) {
            return false;
        }
    }

    if let Some(ref pat) = step.pattern {
        if !simple_wildcard_match(pat, &recorded.target) {
            return false;
        }
    }

    if let Some(ref domain) = step.domain {
        if !simple_wildcard_match(domain, &recorded.target) {
            return false;
        }
    }

    true
}

pub fn simple_wildcard_match(pattern: &str, input: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let parts: Vec<&str> = pattern.split("**").collect();
    if parts.len() == 1 {
        let segments: Vec<&str> = pattern.split('*').collect();
        if segments.len() == 1 {
            return input == pattern;
        }
        let mut pos = 0;
        for (i, seg) in segments.iter().enumerate() {
            if seg.is_empty() {
                continue;
            }
            match input[pos..].find(seg) {
                Some(found) => {
                    if i == 0 && found != 0 {
                        return false;
                    }
                    pos += found + seg.len();
                }
                None => return false,
            }
        }
        return true;
    }

    for part in &parts {
        if part.is_empty() {
            continue;
        }
        if !input.contains(part) {
            return false;
        }
    }
    true
}
