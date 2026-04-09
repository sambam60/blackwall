use crate::envelope::{Action, Decision, EscalationContext, ToolCategory};
use crate::policy::ScoringConfig;
use std::collections::{HashMap, VecDeque};

pub struct RiskScorer {
    config: ScoringConfig,
    window: VecDeque<u32>,
    cumulative: u32,
    label_matchers: HashMap<String, Vec<String>>,
}

impl RiskScorer {
    pub fn new(config: ScoringConfig, labels: &HashMap<String, Vec<String>>) -> Self {
        Self {
            window: VecDeque::with_capacity(config.window),
            cumulative: 0,
            config,
            label_matchers: labels.clone(),
        }
    }

    pub fn score_action(&mut self, action: &Action) -> u32 {
        let score = self.compute_score(action);

        self.window.push_back(score);
        self.cumulative += score;

        while self.window.len() > self.config.window {
            if let Some(old) = self.window.pop_front() {
                self.cumulative = self.cumulative.saturating_sub(old);
            }
        }

        score
    }

    pub fn cumulative_score(&self) -> u32 {
        self.cumulative
    }

    pub fn check_thresholds(&self, _latest_score: u32) -> Option<Decision> {
        if self.cumulative >= self.config.thresholds.halt {
            return Some(Decision::Deny {
                reason: format!(
                    "cumulative risk score {} exceeds halt threshold {}",
                    self.cumulative, self.config.thresholds.halt
                ),
                rule: "scoring.thresholds.halt".into(),
            });
        }

        if self.cumulative >= self.config.thresholds.pause {
            return Some(Decision::Pause {
                reason: format!(
                    "cumulative risk score {} exceeds pause threshold {}",
                    self.cumulative, self.config.thresholds.pause
                ),
                context: EscalationContext {
                    what_happened: "risk score accumulation".into(),
                    why_flagged: format!(
                        "cumulative score {} >= pause threshold {}",
                        self.cumulative, self.config.thresholds.pause
                    ),
                    risk_score: self.cumulative,
                },
            });
        }

        None
    }

    fn compute_score(&self, action: &Action) -> u32 {
        let base_key = format!("{}.{}", action.tool, action.operation);

        if let Some(&weight) = self.config.weights.get(&base_key) {
            return weight;
        }

        let label = self.classify_target(action);
        let label_key = format!("{}.{}.{}", action.tool, action.operation, label);
        if let Some(&weight) = self.config.weights.get(&label_key) {
            return weight;
        }

        match (&action.tool, label.as_str()) {
            (ToolCategory::Filesystem, "secret") => 30,
            (ToolCategory::Filesystem, "sensitive") => 10,
            (ToolCategory::Process, _) => 20,
            (ToolCategory::Network, _) => 5,
            _ => 0,
        }
    }

    fn classify_target(&self, action: &Action) -> String {
        for (label, patterns) in &self.label_matchers {
            for pattern in patterns {
                if crate::pattern::simple_wildcard_match(pattern, &action.target) {
                    return label.clone();
                }
            }
        }
        match &action.tool {
            ToolCategory::Network => "unknown".into(),
            ToolCategory::Shell => "unknown".into(),
            _ => "normal".into(),
        }
    }
}
