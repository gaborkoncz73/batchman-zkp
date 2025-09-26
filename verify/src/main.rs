use ark_groth16::Groth16;
use ark_bn254::Bn254;
use anyhow::Result;
use ark_snark::SNARK;

fn main() -> Result<()> {
    let verify_input: common::VerifyInput = common::load_input("verify.json")?;

    // Proof verifikálás
    let result = Groth16::<Bn254>::verify(&verify_input.vk, &verify_input.public_inputs, &verify_input.proof).unwrap();
    println!("Proof verified? {}", result);
    Ok(())
}
