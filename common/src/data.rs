use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleTemplateFile {
    pub predicates: Vec<PredicateTemplate>,
    pub facts: Vec<FactTemplate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PredicateTemplate {
    pub name: String,
    pub arity: usize,
    pub clauses: Vec<ClauseTemplate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClauseTemplate {
    pub children: Vec<ChildSig>,
    pub equalities: Vec<Equality>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Equality {
    pub left: TermRef,
    pub right: TermRef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TermRef {
    pub node: usize, // 0 = head, 1..N = child index
    pub arg: usize,  // argument position within that node
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChildSig {
    pub name: String,
    pub arity: usize, // later args check
}

#[derive(Debug, Deserialize, Serialize)]
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
