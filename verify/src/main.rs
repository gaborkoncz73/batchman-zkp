mod reader;

use std::fs;
use std::sync::Arc;
use rayon::prelude::*;
use common::{data::RuleTemplateFileFp, *};
use halo2_proofs::{
    pasta::{EqAffine, Fp},
    plonk::{keygen_vk, verify_proof, SingleVerifier, VerifyingKey},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};

use common::unification_checker_circuit::UnificationCircuit;
use common::data::UnificationInputFp;
use reader::read_proofs;
fn main() -> anyhow::Result<()> {
    // load proofs
    let proofs: Vec<(Vec<Vec<Fp>>, Vec<u8>)> = read_proofs("unif")?;
    println!("Verifying {} unification proofs", proofs.len());

    let rules_text = fs::read_to_string("input/rules_template.json")?;
    let rules: data::RuleTemplateFile = serde_json::from_str(&rules_text)?;



    let rules_fp = RuleTemplateFileFp::from(&rules);



    // same parameters as proving side

    let params: Params<EqAffine> = Params::new(8);
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
    let proofs: Vec<(Vec<Vec<Fp>>, Vec<u8>)> = read_proofs("unif")?;

    let ok = proofs.par_iter().all(|(inputs, proof)| {
        //let instances: Vec<Vec<Fp>> = inputs.clone();
        //let instance_refs: Vec<Vec<&[Fp]>> =
        //    instances.iter().map(|v| vec![v.as_slice()]).collect();
        let inner: [&[Fp]; 1] = [inputs[0].as_slice()];
        let instance_refs: Vec<&[&[Fp]]> = vec![&inner];

        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
        let strategy = SingleVerifier::new(params.as_ref());
        verify_proof(params.as_ref(), vk.as_ref(), strategy, &instance_refs, &mut transcript).is_ok()
    });
    if ok {
        println!("All proofs verified successfully!");
    } else {
        println!("Some proofs failed verification!");
    }

    Ok(())
}
