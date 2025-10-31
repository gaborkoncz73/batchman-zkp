use serde::{Deserialize, Serialize};
use halo2_proofs::pasta::Fp;
use crate::utils_2::common_helpers::{MAX_ARITY, MAX_CHILDREN, MAX_CLAUSES, MAX_EQUALITIES, MAX_FACTS, MAX_PRED_LIST, MAX_PREDICATES, to_fp_value};

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
    pub subtree: Vec<ProofNode>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnificationInputFp {
    pub goal_name: Vec<TermFp>,
    pub subtree_goals: Vec<Vec<TermFp>>,
}
impl Default for UnificationInputFp {
    fn default() -> Self {
        Self {
            goal_name: vec![TermFp::default(); MAX_PRED_LIST],
            subtree_goals: vec![vec![TermFp::default(); MAX_PRED_LIST];MAX_ARITY],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TermFp {
    pub name: String,
    pub args: Vec<Vec<String>>,
    pub fact_hashes: String,
}
impl Default for TermFp{
    fn default() -> Self {
        Self {
            name: String::new(),
            args: vec![vec![String::new(); MAX_PRED_LIST];MAX_ARITY], 
            fact_hashes: String::new(), }
    }
}

#[derive(Clone, Debug)]
pub struct RuleTemplateFileFp {
    pub predicates: [PredicateTemplateFp; MAX_PREDICATES],
    pub facts: [FactTemplateFp; MAX_FACTS],
}

impl From<&RuleTemplateFile> for RuleTemplateFileFp {
    fn from(r: &RuleTemplateFile) -> Self {
        let mut preds_fixed = std::array::from_fn(|_| PredicateTemplateFp::default());
        for (i, p) in r.predicates.iter().enumerate().take(MAX_PREDICATES) {
            let mut clauses_fixed = std::array::from_fn(|_| ClauseTemplateFp::default());
            for (j, c) in p.clauses.iter().enumerate().take(MAX_CLAUSES) {
                clauses_fixed[j] = ClauseTemplateFp::from(c);
            }
            preds_fixed[i] = PredicateTemplateFp {
                name: to_fp_value(&p.name),
                arity: Fp::from(p.arity as u64),
                clauses: clauses_fixed,
            };
        }

        let mut facts_fixed = std::array::from_fn(|_| FactTemplateFp::default());
        for (i, f) in r.facts.iter().enumerate().take(MAX_FACTS) {
            facts_fixed[i] = FactTemplateFp {
                name: to_fp_value(&f.name),
                arity: Fp::from(f.arity as u64),
            };
        }

        Self {
            predicates: preds_fixed,
            facts: facts_fixed,
        }
    }
    
}

impl RuleTemplateFileFp {
    /// Flatten all rules and facts into a single Vec<Fp>
    pub fn to_flat_vec(&self) -> Vec<Fp> {
        let mut v = Vec::new();

        // Iterate over predicates
        for pred in &self.predicates {
            v.push(pred.name);
            v.push(pred.arity);

            // Iterate over clauses
            for clause in &pred.clauses {
                // Children
                for ch in &clause.children {
                    v.push(ch.name);
                    v.push(ch.arity);
                }

                // Equalities
                for eq in &clause.equalities {
                    v.push(eq.left.node);
                    v.push(eq.left.arg);
                    v.push(eq.right.node);
                    v.push(eq.right.arg);
                }
            }
        }

        // Iterate over facts
        for fact in &self.facts {
            v.push(fact.name);
            v.push(fact.arity);
        }
        v
    }
}

impl From<&ClauseTemplate> for ClauseTemplateFp {
    fn from(c: &ClauseTemplate) -> Self {
        let mut children_fixed = std::array::from_fn(|_| ChildSigFp::default());
        for (i, ch) in c.children.iter().enumerate().take(MAX_CHILDREN) {
            children_fixed[i] = ChildSigFp {
                name: to_fp_value(&ch.name),
                arity: Fp::from(ch.arity as u64),
            };
        }

        let mut eq_fixed = std::array::from_fn(|_| EqualityFp::default());
        for (i, eq) in c.equalities.iter().enumerate().take(MAX_EQUALITIES) {
            eq_fixed[i] = EqualityFp {
                left: TermRefFp {
                    node: Fp::from(eq.left.node as u64),
                    arg: Fp::from(eq.left.arg as u64),
                },
                right: TermRefFp {
                    node: Fp::from(eq.right.node as u64),
                    arg: Fp::from(eq.right.arg as u64),
                },
            };
        }

        ClauseTemplateFp { children: children_fixed, equalities: eq_fixed }
    }
}

#[derive(Clone, Debug)]
pub struct PredicateTemplateFp {
    pub name: Fp,
    pub arity: Fp,
    pub clauses: [ClauseTemplateFp; MAX_CLAUSES],
}

impl Default for PredicateTemplateFp {
    fn default() -> Self {
        Self {
            name: Fp::zero(),
            arity: Fp::zero(),
            clauses: std::array::from_fn(|_| ClauseTemplateFp::default()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClauseTemplateFp {
    pub children: [ChildSigFp; MAX_CHILDREN],
    pub equalities: [EqualityFp; MAX_EQUALITIES],
}

impl Default for ClauseTemplateFp {
    fn default() -> Self {
        Self {
            children: std::array::from_fn(|_| ChildSigFp::default()),
            equalities: std::array::from_fn(|_| EqualityFp::default()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChildSigFp {
    pub name: Fp,
    pub arity: Fp,
}
impl Default for ChildSigFp {
    fn default() -> Self {
        Self { name: Fp::zero(), arity: Fp::zero() }
    }
}

#[derive(Clone, Debug)]
pub struct EqualityFp {
    pub left: TermRefFp,
    pub right: TermRefFp,
}
impl Default for EqualityFp {
    fn default() -> Self {
        Self { 
            left: TermRefFp::default(),
            right: TermRefFp::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TermRefFp {
    pub node: Fp,
    pub arg: Fp,
}
impl Default for TermRefFp {
    fn default() -> Self {
        Self { node: Fp::zero(), arg: Fp::zero() }
    }
}

#[derive(Clone, Debug)]
pub struct FactTemplateFp {
    pub name: Fp,
    pub arity: Fp,
}
impl Default for FactTemplateFp {
    fn default() -> Self {
        Self { name: Fp::zero(), arity: Fp::zero() }
    }
}

// Config struct to read the yaml
#[derive(Debug, Deserialize)]
pub struct Config {
    pub predicate: String,
    pub args: Vec<String>,
    pub salt: String,
}