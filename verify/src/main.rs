mod reader;

use std::fs;
use std::sync::Arc;
use rayon::prelude::*;
use common::*;
use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_vk, verify_proof, SingleVerifier, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};

use common::unification_checker_circuit::UnificationCircuit;
use common::data::UnificationInput;
use reader::read_proofs;

fn main() -> anyhow::Result<()> {
    // load proofs
    let proofs = read_proofs("unif")?;
    println!("Verifying {} unification proofs", proofs.len());

    let rules_text = fs::read_to_string("input/rules_template2.json")?;
    let rules: data::RuleTemplateFile = serde_json::from_str(&rules_text)?;

    // same parameters as proving side
    let params: Params<EqAffine> = Params::new(5);
    let shape = UnificationCircuit {
        rules: rules.clone(),
        unif: UnificationInput::default(),
    };
    let vk: VerifyingKey<EqAffine> = keygen_vk(&params, &shape)?;
    let params = Arc::new(params);
    let vk = Arc::new(vk);

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
