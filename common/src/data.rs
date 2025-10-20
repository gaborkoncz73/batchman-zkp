use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RuleTemplateFile {
    pub predicates: Vec<PredicateTemplate>,
    pub facts: Vec<FactTemplate>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredicateTemplate {
    pub name: String,
    pub arity: usize,
    pub clauses: Vec<ClauseTemplate>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClauseTemplate {
    pub children: Vec<ChildSig>,
    pub equalities: Vec<Equality>,
}

impl ClauseTemplate {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            equalities: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Equality {
    pub left: TermRef,
    pub right: TermRef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TermRef {
    pub node: usize, // 0 = head, 1..N = child index
    pub arg: usize,  // argument position within that node
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChildSig {
    pub name: String,
    pub arity: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FactTemplate {
    pub name: String,
    pub arity: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ProofNode {
    GoalNode(GoalEntry),
    True(bool),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoalEntry {
    pub goal: String,
    pub goal_term: Term,
    pub goal_unification: Unification,
    pub substitution: Vec<String>,
    pub subtree: Vec<ProofNode>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Term {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Unification {
    pub goal: String,
    pub body: Vec<serde_json::Value>,
}
// ------------------------------------------------------
// 🔹 Flat input struktúra (nem tartalmaz rekurzív subtree-t)
// ------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnificationInput {
    pub goal_name: String,
    pub goal_term_args: Vec<String>,
    pub goal_term_name: String,
    pub unif_body: Vec<String>,          // pl. ["parent(alice,bob)", "ancestor(bob,john)"]
    pub unif_goal: String,
    pub substitution: Vec<String>,       // pl. ["X=bob", "Y=john"]
    pub subtree_goals: Vec<String>,      // pl. ["parent(alice,bob)", "ancestor(bob,john)"]
}

impl Default for UnificationInput {
    fn default() -> Self {
        Self {
            goal_name: String::new(),
            goal_term_args: Vec::new(),
            goal_term_name: String::new(),
            unif_body: Vec::new(),
            unif_goal: String::new(),
            substitution: Vec::new(),
            subtree_goals: Vec::new(),
        }
    }
}

