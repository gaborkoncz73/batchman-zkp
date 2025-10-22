use halo2_proofs::{
    circuit::{Chip, Layouter, Value, AssignedCell},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Selector},
    poly::Rotation,
};

use crate::chips::rlc_chip::{RlcFixedChip, RlcFixedConfig};
use crate::utils_2::common_helpers::{MAX_CANDIDATES, MAX_SIG_TOKENS};

#[derive(Clone, Debug)]
pub struct SigCheckConfig {
    // proof-tree oldali (name,arity) bekötéshez:
    pub sig_name: Column<Advice>,
    pub sig_arity: Column<Advice>,

    // OR-hoz boole flag-ek:
    pub flag: Column<Advice>,       // b_i
    pub q_bool: Column<Fixed>,      // {b*(b-1)=0} selector
    pub q_sum: Selector,            // Σ b_i == 1 csak ott aktív, ahol engedjük
    pub rlc_cfg: RlcFixedConfig,    // a már meglévő RLC chip (α fix)
}

#[derive(Clone, Debug)]
pub struct SigCheckChip {
    cfg: SigCheckConfig,
}

impl Chip<Fp> for SigCheckChip {
    type Config = SigCheckConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.cfg }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl SigCheckChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>, alpha: Fp) -> SigCheckConfig {
        let sig_name  = meta.advice_column();
        let sig_arity = meta.advice_column();
        let flag      = meta.advice_column();
        let q_bool    = meta.fixed_column();
        let q_sum     = meta.selector();

        meta.enable_equality(sig_name);
        meta.enable_equality(sig_arity);
        meta.enable_equality(flag);

        // (1) booleanity gate
        meta.create_gate("flag booleanity", |meta| {
            let q  = meta.query_fixed(q_bool);
            let b  = meta.query_advice(flag, Rotation::cur());
            vec![ q * b.clone() * (b - halo2_proofs::plonk::Expression::Constant(Fp::one())) ]
        });

        // (2) sum flags == 1 gate — csak akkor aktív, ha q_sum bekapcsolt
        meta.create_gate("sum flags == 1", |meta| {
            let q  = meta.query_selector(q_sum);
            let mut sum_expr = meta.query_advice(flag, Rotation::cur());
            for i in 1..MAX_CANDIDATES {
                sum_expr = sum_expr + meta.query_advice(flag, Rotation(i as i32));
            }
            vec![ q * (sum_expr - halo2_proofs::plonk::Expression::Constant(Fp::one())) ]
        });

        let rlc_cfg = RlcFixedChip::configure(meta, alpha);

        SigCheckConfig { sig_name, sig_arity, flag, q_bool, q_sum, rlc_cfg }
    }

    pub fn construct(cfg: SigCheckConfig) -> Self { Self { cfg } }

    /// (name,arity) párokat RLC-be gyűri: domain-szeparátorral: [0xS, name0, ar0, name1, ar1, ...]
    fn fold_sig_list(
        &self,
        mut layouter: impl Layouter<Fp>,
        pairs: &[(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        use halo2_proofs::circuit::Value;
        let rlc = RlcFixedChip::construct(self.cfg.rlc_cfg.clone());
        let sep = Fp::from(0x534947u64); // 'SIG' hex separator

        // 🔹 Minden token EGY régióban
        let tokens: Vec<AssignedCell<Fp, Fp>> = layouter.assign_region(
            || "sig tokens (sep + pairs + pads)",
            |mut region| {
                let mut toks = Vec::new();
                // sep
                let sep_cell = region.assign_advice(
                    || "sep",
                    self.cfg.rlc_cfg.token,
                    0,
                    || Value::known(sep),
                )?;
                toks.push(sep_cell);

                let mut row = 1;
                for (n, a) in pairs.iter() {
                    let n_tok = region.assign_advice(
                        || "name_tok",
                        self.cfg.rlc_cfg.token,
                        row,
                        || n.value().copied(),
                    )?;
                    region.constrain_equal(n_tok.cell(), n.cell())?;
                    row += 1;

                    let a_tok = region.assign_advice(
                        || "arity_tok",
                        self.cfg.rlc_cfg.token,
                        row,
                        || a.value().copied(),
                    )?;
                    region.constrain_equal(a_tok.cell(), a.cell())?;
                    row += 1;

                    toks.push(n_tok);
                    toks.push(a_tok);
                }

                // padding nullákkal
                while toks.len() < MAX_SIG_TOKENS {
                    let pad = region.assign_advice(
                        || "pad",
                        self.cfg.rlc_cfg.token,
                        row,
                        || Value::known(Fp::zero()),
                    )?;
                    toks.push(pad);
                    row += 1;
                }

                Ok::<_, Error>(toks)
            },
        )?;

        // RLC fold (az összes token egyszerre)
        let (combined, _) = rlc.assign_from_cells(
            layouter.namespace(|| "RLC(sig list full)"),
            &tokens,
        )?;

        Ok(combined)
    }

    /// Ellenőrzi, hogy a proof (name,arity) lista ∈ { candidate[i] } halmaznak (OR-tagság)
pub fn check_membership_or(
    &self,
    mut layouter: impl Layouter<Fp>,
    proof_pairs: &[(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)],
    candidate_pairs: &Vec<Vec<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>)>>,
) -> Result<(), Error> {
    use halo2_proofs::circuit::Value;
    let cfg = &self.cfg;

    // --- 1️⃣ proof kombinálása (egyszer)
    let proof_rlc = self.fold_sig_list(layouter.namespace(|| "fold proof sigs"), proof_pairs)?;

    // --- 2️⃣ minden candidate és flag egyetlen régióban
    // --- 1️⃣ proof_rlc előre ---
let proof_rlc = self.fold_sig_list(layouter.namespace(|| "fold proof sigs"), proof_pairs)?;

// --- 2️⃣ cand_rlc-k előre (külön namespace-ekben) ---
let cand_rlcs: Vec<AssignedCell<Fp, Fp>> = candidate_pairs.iter().enumerate()
    .map(|(i, cand)| {
        self.fold_sig_list(layouter.namespace(|| format!("fold cand {}", i)), cand)
    })
    .collect::<Result<Vec<_>, _>>()?;

// --- 3️⃣ most már minden rlc megvan, jöhet a közös régió ---
let flags: Vec<AssignedCell<Fp, Fp>> = layouter.assign_region(
    || "flags + diff constraints",
    |mut region| {
        let mut out = Vec::with_capacity(candidate_pairs.len());

        for (i, cand_rlc) in cand_rlcs.iter().enumerate() {
            let diff_val = proof_rlc.value().zip(cand_rlc.value()).map(|(a,b)| *a - *b);
            let diff_cell = region.assign_advice(
                || format!("diff_{}", i),
                cfg.sig_name,
                i * 3,
                || diff_val,
            )?;

            region.assign_fixed(
                || format!("q_bool_{}", i),
                cfg.q_bool,
                i * 3 + 1,
                || Value::known(Fp::one()),
            )?;

            let b_val = proof_rlc.value().zip(cand_rlc.value())
                .map(|(p,c)| if *p == *c { Fp::one() } else { Fp::zero() });
            let b_i = region.assign_advice(
                || format!("b_{}", i),
                cfg.flag,
                i * 3 + 1,
                || b_val,
            )?;
            out.push(b_i.clone());

            let prod_val = diff_cell.value().zip(b_i.value()).map(|(d,b)| *d * *b);
            let prod_cell = region.assign_advice(
                || format!("prod_{}", i),
                cfg.sig_arity,
                i * 3 + 2,
                || prod_val,
            )?;
            let zero_cell = region.assign_advice(
                || format!("zero_{}", i),
                cfg.sig_arity,
                i * 3 + 3,
                || Value::known(Fp::zero()),
            )?;
            region.constrain_equal(prod_cell.cell(), zero_cell.cell())?;
        }

        Ok(out)
    },
)?;

    // --- 3️⃣ Σ b_i == 1 constraint
    layouter.assign_region(
        || "sum flags == 1 (activated selector)",
        |mut region| {
            cfg.q_sum.enable(&mut region, 0)?;
            for (i, b) in flags.iter().enumerate() {
                let copy = region.assign_advice(
                    || format!("b_copy_{}", i),
                    cfg.flag,
                    i,
                    || b.value().copied(),
                )?;
                region.constrain_equal(copy.cell(), b.cell())?;
            }
            Ok(())
        },
    )?;

    Ok(())
}
}
