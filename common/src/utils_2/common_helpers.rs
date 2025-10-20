use std::hash::{Hash, Hasher};
use blake2::{Blake2s256, Digest};
use halo2_proofs::pasta::group::ff::FromUniformBytes;
use halo2_proofs::pasta::Fp;

use crate::unification_checker_circuit::MAX_DOT_DIM;


pub fn str_to_fp(
    s: &str
) -> Fp
{
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    let v = h.finish();
    Fp::from(v)
}

pub fn pad(
    mut v: Vec<Fp>
) -> Vec<Fp>
{
    let const_col = v.pop().unwrap_or(Fp::one());
    while v.len() < MAX_DOT_DIM - 1 {
        v.push(Fp::zero());
    }
    v.push(const_col);
    v
}

pub fn fs_coeffs(
    seed: &str,
    m: usize
) -> Vec<Fp>
{
    let mut out = Vec::with_capacity(m);
    for i in 0..m {
        let mut h = Blake2s256::new();
        h.update(seed.as_bytes());
        h.update(i.to_le_bytes());
        let hash32 = h.finalize();

        let mut wide = [0u8; 64];
        wide[..32].copy_from_slice(&hash32);

        let fp = <Fp as FromUniformBytes<64>>::from_uniform_bytes(&wide);
        out.push(fp);
    }
    out
}

pub fn compress_rows(
    rows: &[Vec<Fp>],
    r: &[Fp]
) -> Vec<Fp>
{
    if rows.is_empty() {
        // Return empty vector if there are no rows
        return Vec::new();
    }
    let m = rows[0].len();
    let mut c = vec![Fp::zero(); m];
    for (ri, row) in r.iter().zip(rows.iter()) {
        for j in 0..m {
            c[j] += *ri * row[j];
        }
    }
    c
}

pub fn parse_predicate_call(
    s: &str
) -> Option<(String, usize)>
{
    let pos = s.find('(')?;
    let name = s[..pos].to_string();
    let inside = &s[pos + 1..s.len() - 1];
    let argc = if inside.trim().is_empty() {
        0
    } else {
        inside.split(',').count()
    };
    Some((name, argc))
}
