//! Proving and verifying with the real prover (IPA over the Pasta curves):
//! what the `prove` and `verify` binaries do.
//!
//! Both sides build the same circuit shape from the public policy and the
//! public [`Sizes`], so they derive the same keys; the verifier never takes a
//! key from the prover. A proof file ([`ShardProof`]) holds a shard proof and
//! its link commitments; the fact commitments and the trusted roots are
//! public inputs the verifier supplies itself.

use anyhow::{Context, Result, anyhow, bail};
use halo2_poseidon::poseidon::primitives::P128Pow5T3;
use halo2_proofs::halo2curves::pasta::EqAffine;
use halo2_proofs::plonk::{
    ProvingKey, VerifyingKey, create_proof, keygen_pk, keygen_vk, verify_proof,
};
use halo2_proofs::poly::VerificationStrategy;
use halo2_proofs::poly::commitment::ParamsProver;
use halo2_proofs::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
use halo2_proofs::poly::ipa::multiopen::{ProverIPA, VerifierIPA};
use halo2_proofs::poly::ipa::strategy::SingleStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use policy_constraints::{ConstraintSystem, Fp};
use proof_witness::{Shard, TrustedRoots, check_links_between, decimal, field_element};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::branch::BranchTable;
use crate::circuit::{TreeCircuit, link_commitments, root_digests, shard_public};
use crate::rows::minimum_k;

/// The sizes of a shard circuit: node places, incoming and outgoing link
/// places. They are part of the verifying key, so the verifier fixes them
/// (`config/circuit.json`); the prover splits its tree into shards of at
/// most `capacity` nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sizes {
    pub capacity: usize,
    pub incoming: usize,
    pub outgoing: usize,
}

impl Sizes {
    pub fn from_json(source: &str, label: &str) -> Result<Self> {
        let sizes: Self =
            serde_json::from_str(source).with_context(|| format!("{label}: not circuit sizes"))?;
        if sizes.capacity == 0 {
            bail!("{label}: a shard needs at least one node place");
        }
        Ok(sizes)
    }
}

/// The circuit with these sizes and `roots` root places, without a witness:
/// what both sides generate keys from.
pub fn shape(system: &ConstraintSystem, sizes: Sizes, roots: usize) -> TreeCircuit<Fp, P128Pow5T3> {
    TreeCircuit::shape(
        system.layout().clone(),
        BranchTable::from_policy(system),
        roots,
        sizes.capacity,
        sizes.incoming,
        sizes.outgoing,
    )
}

/// The public parameters and the verifying key of a shard circuit.
pub fn verifying_key(
    system: &ConstraintSystem,
    sizes: Sizes,
    roots: usize,
) -> Result<(ParamsIPA<EqAffine>, VerifyingKey<EqAffine>)> {
    let shape = shape(system, sizes, roots);
    // IPA needs no trusted setup: both sides generate the same parameters.
    let params: ParamsIPA<EqAffine> = ParamsIPA::new(minimum_k(&shape));
    let vk = keygen_vk(&params, &shape).map_err(|error| anyhow!("key generation: {error:?}"))?;
    Ok((params, vk))
}

/// The public parameters and the proving key of a shard circuit.
pub struct ShardKey {
    params: ParamsIPA<EqAffine>,
    pk: ProvingKey<EqAffine>,
}

impl ShardKey {
    pub fn generate(system: &ConstraintSystem, sizes: Sizes, roots: usize) -> Result<Self> {
        let (params, vk) = verifying_key(system, sizes, roots)?;
        let pk = keygen_pk(&params, vk, &shape(system, sizes, roots))
            .map_err(|error| anyhow!("key generation: {error:?}"))?;
        Ok(Self { params, pk })
    }

    /// The `k` of the circuit: it has `2^k` rows.
    pub fn k(&self) -> u32 {
        self.pk.get_vk().get_domain().k()
    }
}

/// One shard proof as the verifier receives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardProof {
    /// The shard's index in the plan (for messages only).
    pub shard: usize,
    /// Whether the shard has the roots (the first shard).
    pub root: bool,
    /// The link commitments: the incoming places', then the outgoing
    /// places'; 0 for an unused place.
    pub links: Vec<Fp>,
    pub proof: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofFile {
    schema_version: u32,
    shard: usize,
    root: bool,
    links: Vec<String>,
    proof: String,
}

const PROOF_SCHEMA: u32 = 1;

impl ShardProof {
    pub fn to_json(&self) -> Result<String> {
        let file = ProofFile {
            schema_version: PROOF_SCHEMA,
            shard: self.shard,
            root: self.root,
            links: self.links.iter().map(|value| decimal(*value)).collect(),
            proof: self
                .proof
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        };
        Ok(serde_json::to_string_pretty(&file)? + "\n")
    }

    pub fn from_json(source: &str, label: &str) -> Result<Self> {
        let file: ProofFile =
            serde_json::from_str(source).with_context(|| format!("{label}: not a proof file"))?;
        if file.schema_version != PROOF_SCHEMA {
            bail!(
                "{label}: schema version {} is not {PROOF_SCHEMA}",
                file.schema_version
            );
        }
        let links = file
            .links
            .iter()
            .enumerate()
            .map(|(index, text)| {
                field_element(text).map_err(|message| anyhow!("{label}: link {index}: {message}"))
            })
            .collect::<Result<_>>()?;
        if !file.proof.len().is_multiple_of(2) || !file.proof.is_ascii() {
            bail!("{label}: the proof is not hexadecimal");
        }
        let proof = (0..file.proof.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&file.proof[at..at + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|_| anyhow!("{label}: the proof is not hexadecimal"))?;
        Ok(Self {
            shard: file.shard,
            root: file.root,
            links,
            proof,
        })
    }
}

/// Proves shard `index` of a plan with `key` (generated for the shard's
/// number of roots). `trusted` orders the roots of the first shard;
/// `commitments` is the public fact list.
pub fn prove_shard(
    system: &ConstraintSystem,
    sizes: Sizes,
    key: &ShardKey,
    index: usize,
    shard: &Shard,
    trusted: &TrustedRoots,
    commitments: &[Fp],
) -> Result<ShardProof> {
    let root = !shard.witness.roots.is_empty();
    let circuit = TreeCircuit::<Fp, P128Pow5T3>::from_shard(
        system,
        shard,
        root.then_some(trusted),
        sizes.capacity,
        sizes.incoming,
        sizes.outgoing,
    )
    .map_err(|error| anyhow!("shard {index}: {error}"))?;
    let links = link_commitments(system.layout(), shard, sizes.incoming, sizes.outgoing);
    let roots = if root {
        root_digests(trusted)
    } else {
        Vec::new()
    };
    let instances = [commitments.to_vec(), roots, links.clone()];
    let columns: Vec<&[Fp]> = instances.iter().map(Vec::as_slice).collect();
    let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
    create_proof::<IPACommitmentScheme<EqAffine>, ProverIPA<EqAffine>, _, _, _, _>(
        &key.params,
        &key.pk,
        &[circuit],
        &[&columns],
        OsRng,
        &mut transcript,
    )
    .map_err(|error| anyhow!("shard {index}: the prover failed: {error:?}"))?;
    Ok(ShardProof {
        shard: index,
        root,
        links,
        proof: transcript.finalize(),
    })
}

/// What an accepted set of shard proofs shows, beyond the theorem of
/// `docs/security-model.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    pub shards: usize,
    pub links: usize,
}

/// The verifying keys of the two kinds of shards, generated by the verifier
/// from the policy and the sizes.
pub struct VerifierKeys {
    sizes: Sizes,
    roots: usize,
    keys: [Option<(ParamsIPA<EqAffine>, VerifyingKey<EqAffine>)>; 2],
}

impl VerifierKeys {
    /// No keys yet: [`verify_proofs_with`] generates each on first use.
    pub fn new(sizes: Sizes, trusted: &TrustedRoots) -> Self {
        Self {
            sizes,
            roots: trusted.goals().count(),
            keys: [None, None],
        }
    }

    /// Generates the key of the root shard and, if `others`, of the others.
    pub fn generate(
        system: &ConstraintSystem,
        sizes: Sizes,
        trusted: &TrustedRoots,
        others: bool,
    ) -> Result<Self> {
        let mut keys = Self::new(sizes, trusted);
        keys.key(system, true)?;
        if others {
            keys.key(system, false)?;
        }
        Ok(keys)
    }

    fn key(
        &mut self,
        system: &ConstraintSystem,
        root: bool,
    ) -> Result<&(ParamsIPA<EqAffine>, VerifyingKey<EqAffine>)> {
        let slot = usize::from(root);
        if self.keys[slot].is_none() {
            let roots = if root { self.roots } else { 0 };
            self.keys[slot] = Some(verifying_key(system, self.sizes, roots)?);
        }
        Ok(self.keys[slot].as_ref().expect("generated above"))
    }
}

/// The verifier: checks a set of shard proofs against the public policy,
/// sizes, trusted roots and fact commitments. Exactly one proof has the
/// roots; every proof verifies under the key the verifier generates itself;
/// and the link commitments of the proofs match (outgoing = incoming as
/// multisets, unused places skipped).
pub fn verify_proofs(
    system: &ConstraintSystem,
    sizes: Sizes,
    trusted: &TrustedRoots,
    commitments: &[Fp],
    proofs: &[ShardProof],
) -> Result<Verified> {
    let mut keys = VerifierKeys::new(sizes, trusted);
    verify_proofs_with(system, &mut keys, trusted, commitments, proofs)
}

/// [`verify_proofs`] with keys generated earlier (or on first use).
pub fn verify_proofs_with(
    system: &ConstraintSystem,
    keys: &mut VerifierKeys,
    trusted: &TrustedRoots,
    commitments: &[Fp],
    proofs: &[ShardProof],
) -> Result<Verified> {
    let sizes = keys.sizes;
    if keys.roots != trusted.goals().count() {
        bail!("the keys were generated for another number of trusted roots");
    }
    let root_proofs = proofs.iter().filter(|proof| proof.root).count();
    if root_proofs != 1 {
        bail!("{root_proofs} proofs claim the trusted roots; exactly one must");
    }
    for proof in proofs {
        if proof.links.len() != sizes.incoming + sizes.outgoing {
            bail!(
                "shard {}: {} link commitments, the circuit has {} places",
                proof.shard,
                proof.links.len(),
                sizes.incoming + sizes.outgoing
            );
        }
        let (params, vk) = keys.key(system, proof.root)?;
        let root_column = if proof.root {
            root_digests(trusted)
        } else {
            Vec::new()
        };
        let instances = [commitments.to_vec(), root_column, proof.links.clone()];
        let columns: Vec<&[Fp]> = instances.iter().map(Vec::as_slice).collect();
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof.proof[..]);
        verify_proof::<IPACommitmentScheme<EqAffine>, VerifierIPA<EqAffine>, _, _, _>(
            params,
            vk,
            SingleStrategy::new(params),
            &[&columns],
            &mut transcript,
        )
        .map_err(|error| {
            anyhow!(
                "shard {}: the proof does not verify: {error:?}",
                proof.shard
            )
        })?;
    }
    let publics: Vec<_> = proofs
        .iter()
        .map(|proof| shard_public(&proof.links, sizes.incoming))
        .collect();
    check_links_between(&publics)?;
    Ok(Verified {
        shards: proofs.len(),
        links: publics.iter().map(|public| public.incoming.len()).sum(),
    })
}
