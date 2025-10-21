mod reader;

use std::fs;
use std::sync::Arc;
use rayon::prelude::*;
use common::{data::RuleTemplateFileFp, utils_2::common_helpers::{cpu_rulehash_first_2, flatten_rule_template_to_fp, hash_rule_template_poseidon, pad_to_const, poseidon_hash_cpu_const}, *};
use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_vk, verify_proof, SingleVerifier, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};

use common::unification_checker_circuit::UnificationCircuit;
use common::data::UnificationInputFp;
use reader::read_proofs;
const L_MAX: usize = 62;
fn main() -> anyhow::Result<()> {
    // load proofs
    let proofs = read_proofs("unif")?;
    println!("Verifying {} unification proofs", proofs.len());

    let rules_text = fs::read_to_string("input/rules_template.json")?;
    let rules: data::RuleTemplateFile = serde_json::from_str(&rules_text)?;

    let res = cpu_rulehash_first_2(&rules);
    println!("Poseidon hash (identical to circuit) = {:?}", res);

    let rules_text2 = fs::read_to_string("input/rules_template2.json")?;
    let rules2: data::RuleTemplateFile = serde_json::from_str(&rules_text2)?;

    let res2 = cpu_rulehash_first_2(&rules2);
    println!("Poseidon hash (identical to circuit) = {:?}", res2);

    let rules_fp = RuleTemplateFileFp::from(&rules);

    let expected = cpu_rulehash_first_2(&rules2);

    let instance = vec![expected]; // 1 publikus input mező
    let instances = vec![instance.as_slice()]; // &[&[Fp]] szintű lista

    // same parameters as proving side
    let params: Params<EqAffine> = Params::new(6);
    let shape = UnificationCircuit {
        rules: rules_fp,
        unif: UnificationInputFp::default(),
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let params = Arc::new(params);
    let vk = Arc::new(vk);

    //let instance = vec![]; // 1 publikus input mező
    //let instances = vec![instance.as_slice()]; // &[&[Fp]] szintű lista

    // parallel verification
    let ok = proofs.par_iter().all(|(_, proof)| {
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
        let strategy = SingleVerifier::new(params.as_ref());
        let instances: Vec<&[&[Fp]]> = vec![&[]];
        verify_proof(params.as_ref(), vk.as_ref(), strategy, &instances, &mut transcript).is_ok()
    });

    if ok {
        println!("All proofs verified successfully!");
    } else {
        println!("Some proofs failed verification!");
    }

    Ok(())
}
