//! The proof tree written by the SWI meta-interpreter (`prolog/meta_interpreter.pl`).
//!
//! ```json
//! [ { "goal": "p(5,a)",
//!     "subtree": [ { "goal": "q(5)", "subtree": [true] } ] } ]
//! ```
//!
//! The file is a list of roots. `true` in a subtree marks a goal that was
//! answered without further goals (a fact or a builtin); it is not a node.

use anyhow::{Context, Result, bail};
use policy_constraints::GroundTerm;
use serde::Deserialize;

use crate::goal::ground_goal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    /// The goal as written in the proof tree.
    pub text: String,
    pub goal: GroundTerm,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// The nodes of a proof tree in pre-order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofTree {
    nodes: Vec<TreeNode>,
    roots: Vec<usize>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Entry {
    Node(RawNode),
    Marker(bool),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNode {
    goal: String,
    subtree: Vec<Entry>,
}

impl ProofTree {
    pub fn from_json(source: &str, label: &str) -> Result<Self> {
        let raw_roots: Vec<RawNode> =
            serde_json::from_str(source).with_context(|| format!("{label}: not a proof tree"))?;
        if raw_roots.is_empty() {
            bail!("{label}: the proof tree has no root");
        }
        let mut tree = Self {
            nodes: Vec::new(),
            roots: Vec::new(),
        };
        for root in &raw_roots {
            let index = tree.add(root, None, label)?;
            tree.roots.push(index);
        }
        Ok(tree)
    }

    fn add(&mut self, raw: &RawNode, parent: Option<usize>, label: &str) -> Result<usize> {
        let index = self.nodes.len();
        let goal = ground_goal(&raw.goal, &format!("{label} node {index}"))?;
        self.nodes.push(TreeNode {
            text: raw.goal.clone(),
            goal,
            parent,
            children: Vec::new(),
        });
        for entry in &raw.subtree {
            match entry {
                Entry::Node(child) => {
                    let child = self.add(child, Some(index), label)?;
                    self.nodes[index].children.push(child);
                }
                Entry::Marker(true) => {}
                Entry::Marker(false) => {
                    bail!("{label} node {index}: `false` in a subtree")
                }
            }
        }
        Ok(index)
    }

    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    pub fn roots(&self) -> &[usize] {
        &self.roots
    }
}
