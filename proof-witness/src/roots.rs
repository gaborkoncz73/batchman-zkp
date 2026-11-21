//! The goals the verifier trusts as roots (`input/trusted_roots.json`).
//!
//! ```json
//! { "root_goals": ["endPrice(931220)"] }
//! ```
//!
//! Other fields of the file (the legacy `schema_version` and `dialect`) are
//! ignored.

use anyhow::{Context, Result, bail};
use policy_constraints::GroundTerm;
use serde::Deserialize;

use crate::goal::ground_goal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedRoots {
    goals: Vec<(String, GroundTerm)>,
}

#[derive(Deserialize)]
struct File {
    root_goals: Vec<String>,
}

impl TrustedRoots {
    pub fn from_json(source: &str, label: &str) -> Result<Self> {
        let file: File = serde_json::from_str(source)
            .with_context(|| format!("{label}: not a trusted-roots file"))?;
        Self::new(&file.root_goals, label)
    }

    pub fn new(texts: &[impl AsRef<str>], label: &str) -> Result<Self> {
        if texts.is_empty() {
            bail!("{label}: there is no trusted root");
        }
        let mut goals: Vec<(String, GroundTerm)> = Vec::with_capacity(texts.len());
        for (index, text) in texts.iter().enumerate() {
            let text = text.as_ref();
            let goal = ground_goal(text, &format!("{label}.root_goals[{index}]"))?;
            if goals.iter().any(|(_, previous)| previous == &goal) {
                bail!("{label}.root_goals[{index}]: {text} is listed twice");
            }
            goals.push((text.to_owned(), goal));
        }
        Ok(Self { goals })
    }

    pub fn goals(&self) -> impl Iterator<Item = &GroundTerm> {
        self.goals.iter().map(|(_, goal)| goal)
    }

    pub fn contains(&self, goal: &GroundTerm) -> bool {
        self.goals().any(|trusted| trusted == goal)
    }
}
