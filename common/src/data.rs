use serde::{Deserialize, Serialize};
use halo2_proofs::pasta::Fp;
use crate::utils_2::common_helpers::{MAX_ARITY, MAX_PRED_LIST, to_fp_value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RuleTemplateFile {
    pub predicates: Vec<PredicateTemplate>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredicateTemplate {
    pub name: String,
    pub arity: usize,
    pub clauses: Vec<ClauseTemplate>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClauseTemplate {
    pub children: Vec<Vec<ChildSig>>,
    pub equalities: Vec<Equality>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TermSide {
    Ref(TermRefComplex),
    Value(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Equality {
    pub left: TermSide,
    pub right: TermSide,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TermRefComplex {
    pub children_node_list: usize,
    pub predicate: usize,
    pub arg: usize,
    pub list_index: usize,
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


#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct TermFp {
    pub name: Fp,
    pub args: Vec<Vec<Fp>>,
    pub fact_hashes: Fp,
}
impl Default for TermFp{
    fn default() -> Self {
        Self {
            name: Fp::zero(),
            args: vec![vec![Fp::zero(); MAX_PRED_LIST];MAX_ARITY], 
            fact_hashes: Fp::zero(), }
    }
}

#[derive(Clone, Debug)]
pub struct RuleTemplateFileFp {
    pub predicates: Vec<PredicateTemplateFp>,
}

impl From<&RuleTemplateFile> for RuleTemplateFileFp {
    fn from(r: &RuleTemplateFile) -> Self {
        let predicates = r.predicates.iter().map(|p| {
            PredicateTemplateFp {
                name: to_fp_value(&p.name),
                arity: Fp::from(p.arity as u64),
                clauses: p.clauses.iter().map(|c| ClauseTemplateFp::from(c)).collect(),
            }
        }).collect();

        Self { predicates }
    }
}

#[derive(Clone, Debug)]
pub struct PredicateTemplateFp {
    pub name: Fp,
    pub arity: Fp,
    pub clauses: Vec<ClauseTemplateFp>,
}

impl From<&ClauseTemplate> for ClauseTemplateFp {
    fn from(c: &ClauseTemplate) -> Self {
        let children = c.children.iter().map(|row| {
            row.iter().map(|ch| ChildSigFp {
                name: to_fp_value(&ch.name),
                arity: Fp::from(ch.arity as u64),
            }).collect()
        }).collect();

        let equalities = c.equalities.iter().map(|eq| {
            EqualityFp {
                left: TermSideFp::from(&eq.left),
                right: TermSideFp::from(&eq.right),
            }
        }).collect();

        Self { children, equalities }
    }
}
impl Default for PredicateTemplateFp {
    fn default() -> Self {
        Self {
            name: Fp::zero(),
            arity: Fp::zero(),
            clauses: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClauseTemplateFp {
    pub children: Vec<Vec<ChildSigFp>>,
    pub equalities: Vec<EqualityFp>,
}

#[derive(Clone, Debug)]
pub struct ChildSigFp {
    pub name: Fp,
    pub arity: Fp,
}

#[derive(Clone, Debug)]
pub struct EqualityFp {
    pub left: TermSideFp,
    pub right: TermSideFp,
}

#[derive(Clone, Debug)]
pub enum TermSideFp {
    Ref(TermRefFp),
    Value(Fp),
}

impl From<&TermSide> for TermSideFp {
    fn from(ts: &TermSide) -> Self {
        match ts {
            TermSide::Ref(r) => TermSideFp::Ref(TermRefFp {
                children_node_list: Fp::from(r.children_node_list as u64),
                predicate: Fp::from(r.predicate as u64),
                arg: Fp::from(r.arg as u64),
                list_index: Fp::from(r.list_index as u64),
            }),
            TermSide::Value(v) => TermSideFp::Value(to_fp_value(v)),
        }
    }
}


#[derive(Clone, Debug)]
pub struct TermRefFp {
    pub children_node_list: Fp,
    pub predicate: Fp,
    pub arg: Fp,
    pub list_index: Fp,
}


// Config struct to read the yaml
#[derive(Debug, Deserialize)]
pub struct FactEntry {
    pub predicate: String,
    pub args: Vec<Vec<String>>, // minden arg stringként jön a YAML-ből
    pub salt: String,
}
