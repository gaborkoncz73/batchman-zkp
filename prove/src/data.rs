use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RuleTemplateFile {
    pub predicates: Vec<PredicateTemplate>,
    pub facts: Vec<FactTemplate>,
}

#[derive(Debug, Deserialize)]
pub struct PredicateTemplate {
    pub name: String,
    pub arity: usize,
    pub clauses: Vec<ClauseTemplate>,
}

#[derive(Debug, Deserialize)]
pub struct ClauseTemplate {
    pub children: Vec<ChildSig>,
    //pub equalities: Vec<String>, // later args check
}

#[derive(Debug, Deserialize)]
pub struct ChildSig {
    pub name: String,
    //pub arity: usize, // later args check
}

#[derive(Debug, Deserialize)]
pub struct FactTemplate {
    pub name: String,
    pub arity: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProofNode {
    GoalNode(GoalEntry),
    True(bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoalEntry {
    pub goal: String,
    pub goal_term: Term,
    pub goal_unification: Unification,
    pub substitution: Vec<String>,
    pub subtree: Vec<ProofNode>,
}

#[derive(Debug, Serialize,  Deserialize)]
pub struct Term {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unification {
    pub goal: String,
    pub body: Vec<serde_json::Value>,
}
