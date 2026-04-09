use crate::envelope::{Decision, EscalationContext};
use crate::policy::CircuitBreakerConfig;
use std::time::Instant;

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    denial_count: u32,
    total_actions: u64,
    actions_this_minute: u32,
    minute_start: Instant,
    tripped: bool,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            denial_count: 0,
            total_actions: 0,
            actions_this_minute: 0,
            minute_start: Instant::now(),
            tripped: false,
        }
    }

    pub fn record_action(&mut self) {
        self.total_actions += 1;

        if self.minute_start.elapsed().as_secs() >= 60 {
            self.actions_this_minute = 0;
            self.minute_start = Instant::now();
        }
        self.actions_this_minute += 1;
    }

    pub fn record_denial(&mut self) {
        self.denial_count += 1;
    }

    pub fn check(&self) -> Option<Decision> {
        if self.tripped {
            return Some(Decision::Deny {
                reason: "circuit breaker tripped — session halted".into(),
                rule: "circuit_breakers.tripped".into(),
            });
        }

        if self.denial_count >= self.config.max_denials_before_halt {
            return Some(Decision::Deny {
                reason: format!(
                    "{} denials exceeds threshold of {}",
                    self.denial_count, self.config.max_denials_before_halt
                ),
                rule: "circuit_breakers.max_denials_before_halt".into(),
            });
        }

        if self.total_actions >= self.config.max_actions_per_session {
            return Some(Decision::Deny {
                reason: format!(
                    "{} total actions exceeds session limit of {}",
                    self.total_actions, self.config.max_actions_per_session
                ),
                rule: "circuit_breakers.max_actions_per_session".into(),
            });
        }

        if self.actions_this_minute >= self.config.max_actions_per_minute {
            return Some(Decision::Pause {
                reason: format!(
                    "{} actions/min exceeds rate limit of {}",
                    self.actions_this_minute, self.config.max_actions_per_minute
                ),
                context: EscalationContext {
                    what_happened: "action rate limit exceeded".into(),
                    why_flagged: format!(
                        "{}/min vs {}/min limit",
                        self.actions_this_minute, self.config.max_actions_per_minute
                    ),
                    risk_score: 0,
                },
            });
        }

        None
    }

    pub fn trip(&mut self) {
        self.tripped = true;
    }

    pub fn reset(&mut self) {
        self.tripped = false;
        self.denial_count = 0;
        self.actions_this_minute = 0;
    }

    pub fn is_tripped(&self) -> bool {
        self.tripped
    }
}
