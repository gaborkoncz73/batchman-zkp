use serde::{Deserialize, Serialize};
use halo2_proofs::pasta::Fp;
use crate::utils_2::common_helpers::{to_fp_value, MAX_ARITY};

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
/*#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnificationInput {
    pub goal_name: String,
    pub goal_term_args: Vec<String>,
    pub goal_term_name: String,
    pub unif_body: Vec<String>,          // pl. ["parent(alice,bob)", "ancestor(bob,john)"]
    pub unif_goal: String,
    pub substitution: Vec<String>,       // pl. ["X=bob", "Y=john"]
    pub subtree_goals: Vec<String>,      // pl. ["parent(alice,bob)", "ancestor(bob,john)"]
}*/

#[derive(Clone, Debug)]
pub struct UnificationInputFp {
    pub goal_name: Fp,
    pub goal_term_args: Vec<Fp>,
    pub goal_term_name: Fp,
    pub unif_body: Vec<Fp>,
    pub unif_goal: Fp,
    pub substitution: Vec<Fp>,
    pub subtree_goals: Vec<Fp>,
}

impl Default for UnificationInputFp {
    fn default() -> Self {
        Self {
            goal_name: Fp::zero(),
            goal_term_args: vec![Fp::zero(); MAX_ARITY],
            goal_term_name: Fp::zero(),
            unif_body: Vec::new(),
            unif_goal: Fp::zero(),
            substitution: Vec::new(),
            subtree_goals: Vec::new(),
        }
    }
}


#[derive(Clone, Debug)]
pub struct RuleTemplateFileFp {
    pub predicates: Vec<PredicateTemplateFp>,
    pub facts: Vec<FactTemplateFp>,
}

impl From<&RuleTemplateFile> for RuleTemplateFileFp {
    fn from(r: &RuleTemplateFile) -> Self {
        RuleTemplateFileFp {
            predicates: r.predicates.iter().map(|p| PredicateTemplateFp {
                name: to_fp_value(&p.name),
                arity: Fp::from(p.arity as u64),
                clauses: p.clauses.iter().map(|c| ClauseTemplateFp {
                    children: c.children.iter().map(|ch| ChildSigFp {
                        name: to_fp_value(&ch.name),
                        arity: Fp::from(ch.arity as u64),
                    }).collect(),
                    equalities: c.equalities.iter().map(|eq| EqualityFp {
                        left: TermRefFp {
                            node: Fp::from(eq.left.node as u64),
                            arg: Fp::from(eq.left.arg as u64),
                        },
                        right: TermRefFp {
                            node: Fp::from(eq.right.node as u64),
                            arg: Fp::from(eq.right.arg as u64),
                        },
                    }).collect(),
                }).collect(),
            }).collect(),
            facts: r.facts.iter().map(|f| FactTemplateFp {
                name: to_fp_value(&f.name),
                arity: Fp::from(f.arity as u64),
            }).collect(),
        }
    }
}


#[derive(Clone, Debug)]
pub struct PredicateTemplateFp {
    pub name: Fp,
    pub arity: Fp,
    pub clauses: Vec<ClauseTemplateFp>,
}

#[derive(Clone, Debug)]
pub struct ClauseTemplateFp {
    pub children: Vec<ChildSigFp>,
    pub equalities: Vec<EqualityFp>,
}

#[derive(Clone, Debug)]
pub struct ChildSigFp {
    pub name: Fp,
    pub arity: Fp,
}

#[derive(Clone, Debug)]
pub struct EqualityFp {
    pub left: TermRefFp,
    pub right: TermRefFp,
}

#[derive(Clone, Debug)]
pub struct TermRefFp {
    pub node: Fp,
    pub arg: Fp,
}

#[derive(Clone, Debug)]
pub struct FactTemplateFp {
    pub name: Fp,
    pub arity: Fp,
}

