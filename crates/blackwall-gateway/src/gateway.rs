use blackwall_core::audit::{AuditEntry, AuditLog};
use blackwall_core::engine::PolicyEngine;
use blackwall_core::envelope::{ActionEnvelope, Decision};
use std::time::Instant;

pub struct BlackwallGateway {
    engine: PolicyEngine,
    audit: AuditLog,
}

impl BlackwallGateway {
    pub fn new(engine: PolicyEngine, audit: AuditLog) -> Self {
        Self { engine, audit }
    }

    pub fn process(&mut self, envelope: &ActionEnvelope) -> Decision {
        let start = Instant::now();
        let decision = self.engine.evaluate(envelope);
        let latency_us = start.elapsed().as_micros() as u64;

        let entry = AuditEntry::from_evaluation(envelope, &decision, latency_us);

        eprintln!("{}", entry.display_line());

        if let Err(e) = self.audit.log(&entry) {
            eprintln!("  \x1b[31maudit write failed: {}\x1b[0m", e);
        }

        decision
    }

    pub fn action_count(&self) -> u64 {
        self.engine.action_count()
    }

    pub fn risk_score(&self) -> u32 {
        self.engine.risk_score()
    }

    pub fn session_id(&self) -> &str {
        self.audit.session_id()
    }
}
