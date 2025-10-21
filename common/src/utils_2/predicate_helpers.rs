
use halo2_proofs::pasta::{group::ff::FromUniformBytes, Fp};
use blake2::{Blake2s256, Digest};
use halo2_proofs::arithmetic::Field;

use crate::{data::{ClauseTemplate, PredicateTemplate, RuleTemplateFile}, utils_2::common_helpers::parse_predicate_call};

pub fn get_matching_predicates(
    predicates: &[PredicateTemplate],
    goal_term_name: &str,
    goal_term_args: &[String],
) -> Result<Vec<PredicateTemplate>, halo2_proofs::plonk::Error> 
{
    let mut matchers: Vec<PredicateTemplate> = Vec::new();

    for pred in predicates.iter()
    {
        if pred.name == goal_term_name && pred.arity == goal_term_args.len()
        {
            matchers.push(pred.clone());
        }
    }

    return Ok(matchers);
}

/*pub fn get_matching_structure_and_vectors(
    unification: &UnificationInput,
    rules: &RuleTemplateFile,
)-> Result<(ClauseTemplate,Fp,Fp), halo2_proofs::plonk::Error> 
{
    let matchers = get_matching_predicates(
        &rules.predicates,
        &unification.goal_term_name,
        &unification.goal_term_args
    )?;

    for pred in matchers.iter()
    {
        for clause in pred.clauses.iter(){

            let rule_pred = blake_hash_predicate(&pred.name, &pred.arity, &clause);
            let tree_pred = blake_hash_goal_unification(&unification.goal_name, &unification.unif_body);
            let dot_debug = rule_pred-tree_pred;

            if dot_debug.is_zero_vartime() {
                return Ok((clause.clone(), rule_pred, tree_pred));
            }
        }
    }
    return Ok((
        ClauseTemplate::new(),
        Fp::zero(),
        Fp::zero(),
    ));
    //Err(halo2_proofs::plonk::Error::Synthesis)
}*/

// Mock hash later will be swapped with better one
pub fn blake_hash_predicate(
    pred_name: &str,
    pred_arity: &usize,
    clause: &ClauseTemplate
) -> Fp 
{
    let mut h = Blake2s256::new();

    h.update(pred_name.as_bytes());
    h.update(pred_arity.to_le_bytes());

    for child in &clause.children {
        h.update(child.name.as_bytes());
        h.update(child.arity.to_le_bytes());
    }

    let hash32 = h.finalize();
    let mut wide = [0u8; 64];
    wide[..32].copy_from_slice(&hash32);

    <Fp as FromUniformBytes<64>>::from_uniform_bytes(&wide)
}

pub fn blake_hash_goal_unification(
    goal: &str,
    body: &[String]
) -> Fp
{
    let mut h = Blake2s256::new();

    let (goal_name, goal_arity) = parse_predicate_call(goal).unwrap();

    h.update(goal_name.as_bytes());
    h.update(goal_arity.to_le_bytes());

    for pred in body {
        let (name, arity) = parse_predicate_call(pred).unwrap();
        h.update(name.as_bytes());
        h.update(arity.to_le_bytes());
    }

    let hash32 = h.finalize();
    let mut wide = [0u8; 64];
    wide[..32].copy_from_slice(&hash32);

    <Fp as FromUniformBytes<64>>::from_uniform_bytes(&wide)
}