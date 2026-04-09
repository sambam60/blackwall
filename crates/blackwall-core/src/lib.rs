pub mod audit;
pub mod circuit;
pub mod engine;
pub mod envelope;
pub mod pattern;
pub mod policy;
pub mod scoring;

pub use engine::PolicyEngine;
pub use envelope::{Action, ActionEnvelope, Decision, EscalationContext, ToolCategory};
pub use policy::Policy;

#[derive(Debug, thiserror::Error)]
pub enum BlackwallError {
    #[error("invalid glob pattern '{pattern}': {source}")]
    InvalidGlob {
        pattern: String,
        #[source]
        source: globset::Error,
    },

    #[error("invalid policy: {0}")]
    InvalidPolicy(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Yaml(#[from] serde_yaml::Error),
}
