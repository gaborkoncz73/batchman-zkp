use halo2_proofs::pasta::Fp;
use crate::utils_2::common_helpers::str_to_fp;



// Constructs the tuples
// Used to check that the arguments in the goal name and in the term args list are the same
// Also checks that the unification body elements are identical to the subtree elements
pub fn build_consistency_pairs(
    goal_name: &str,
    goal_term_args: &[String],
    unif_body: &[String],
    subtree_goals: &[String]
) -> Result<Vec<(Fp, Fp)>, halo2_proofs::plonk::Error>
{
    let mut all_pairs: Vec<(Fp, Fp)> = Vec::new();
    
    // goal_name vs goal_term_args
    let goal_term_pairs: Vec<(Fp, Fp)> = extract_args(&goal_name)
        .into_iter() // itt fontos a sorrend, ne legyen unordered
        .zip(goal_term_args.iter())
        .map(|(a, b)| (str_to_fp(&a), str_to_fp(b)))
        .collect();

    // unif_body vs subtree_goals
    let body_subtree_pairs: Vec<(Fp, Fp)> = unif_body
        .iter()
        .zip(subtree_goals.iter())
        .map(|(body_str, subtree_str)| (str_to_fp(body_str), str_to_fp(subtree_str)))
        .collect();

    // chain into a list
    all_pairs.extend(goal_term_pairs);
    all_pairs.extend(body_subtree_pairs);

    return Ok(all_pairs);
}

// From the complete string it returns the arguments taken out from the ()s
fn extract_args(goal_str: &str) -> Vec<String> {
    if let Some(start) = goal_str.find('(') {
        if let Some(end) = goal_str.find(')') {
            return goal_str[start + 1..end] 
                .split(',')                  
                .map(|s| s.trim().to_string()) 
                .filter(|s| !s.is_empty())     
                .collect();
        }
    }
    vec![]
}
