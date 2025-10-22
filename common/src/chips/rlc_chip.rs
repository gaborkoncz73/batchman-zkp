use halo2_proofs::{
    circuit::{Chip, Layouter, Value, AssignedCell},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};

use crate::{chips::rlc_chip, data::TermFp, utils_2::common_helpers::{MAX_ARITY, MAX_PAIRS, MAX_SUBTREE_LEN}};

#[derive(Clone, Debug)]
pub struct RlcFixedConfig {
    pub token: Column<Advice>,  // input tokenek: [name, arg0, arg1, ...]
    pub acc: Column<Advice>,    // akkumulátor
    pub q: Column<Fixed>,       // selector
    pub alpha: Fp,              // FIX konstans α
}

pub struct RlcFixedChip {
    cfg: RlcFixedConfig,
}

impl Chip<Fp> for RlcFixedChip {
    type Config = RlcFixedConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.cfg }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl RlcFixedChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>, alpha: Fp) -> RlcFixedConfig {
        let token  = meta.advice_column();
        let acc    = meta.advice_column();
        let q      = meta.fixed_column();

        meta.enable_equality(token);
        meta.enable_equality(acc);

        // Gate: acc_{i+1} = acc_i * α + token_i
        meta.create_gate("RLC fixed-alpha fold", |meta| {
            let q     = meta.query_fixed(q);
            let acc_c = meta.query_advice(acc, Rotation::cur());
            let acc_n = meta.query_advice(acc, Rotation::next());
            let tok   = meta.query_advice(token, Rotation::cur());
            let alpha = halo2_proofs::plonk::Expression::Constant(alpha);
            vec![ q * (acc_n - (acc_c * alpha + tok)) ]
        });

        RlcFixedConfig { token, acc, q, alpha }
    }

    pub fn construct(cfg: RlcFixedConfig) -> Self { Self { cfg } }

    /// tokens = [name, arg0, arg1, ...] → returns combined cell (acc_last)
    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        tokens: &[Value<Fp>],
    ) -> Result<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>), Error> {
        assert!(!tokens.is_empty());
        let cfg = self.config();
        let n = tokens.len();

        let (last, token_cells) = layouter.assign_region(
            || "RLC single region (Value version)",
            |mut region| {
                // q / q_last
                for r in 0..=n {
                    let q_val = if r < n { Fp::one() } else { Fp::zero() };
                    region.assign_fixed(|| "q", cfg.q, r, || Value::known(q_val))?;
                }

                // acc_0
                let mut acc_val = Value::known(Fp::zero());
                region.assign_advice(|| "acc_0", cfg.acc, 0, || acc_val)?;

                // token-ek + fold
                let mut token_cells = Vec::with_capacity(n);
                for i in 0..n {
                    let t_cell = region.assign_advice(|| "token", cfg.token, i, || tokens[i].clone())?;
                    token_cells.push(t_cell);
                    acc_val = acc_val.zip(tokens[i]).map(|(acc, t)| acc * cfg.alpha + t);
                    region.assign_advice(|| "acc", cfg.acc, i + 1, || acc_val)?;
                }

                // output
                let last_cell = region.assign_advice(|| "acc_last_out", cfg.acc, n, || acc_val)?;
                Ok((last_cell, token_cells))
            },
        )?;

        Ok((last, token_cells))
    }
    /// RLC fold közvetlenül létező AssignedCell-ekből.
/// A token oszlopba tett értékeket _constrain_equal_-lal visszakötjük az input cellákhoz.
pub fn assign_from_cells(
    &self,
    mut layouter: impl Layouter<Fp>,
    inputs: &[AssignedCell<Fp, Fp>],
) -> Result<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>), Error> {
    use halo2_proofs::circuit::Value;
    let cfg = self.config();
    let n = inputs.len();

    layouter.assign_region(
        || "RLC(from cells)",
        |mut region| {
            // q selector
            for r in 0..=n {
                let qv = if r < n { Fp::one() } else { Fp::zero() };
                region.assign_fixed(|| "q", cfg.q, r, || Value::known(qv))?;
            }

            // acc_0 = 0
            let mut acc = Value::known(Fp::zero());
            region.assign_advice(|| "acc_0", cfg.acc, 0, || acc)?;

            // token-ek + fold, ÉS a token visszakötése az input cellához
            let mut token_cells = Vec::with_capacity(n);
            for i in 0..n {
                let t = region.assign_advice(
                    || "token",
                    cfg.token,
                    i,
                    || inputs[i].value().copied(),
                )?;
                // ← fontos: összekötjük a token cellát az eredeti inputtal
                region.constrain_equal(t.cell(), inputs[i].cell())?;

                token_cells.push(t);
                acc = acc.zip(inputs[i].value()).map(|(a, v)| a * cfg.alpha + v);
                region.assign_advice(|| "acc", cfg.acc, i + 1, || acc)?;
            }

            let last = region.assign_advice(|| "acc_last", cfg.acc, n, || acc)?;
            Ok((last, token_cells))
        }
    )
}
/// Páronkénti összehasonlítás ugyanazzal az RLC-vel: [name, args...] ⇢ fold, majd equality.
pub fn cmp_term_lists_pairwise_with_rlc_cells(
    &self,
    mut layouter: impl Layouter<Fp>,
    left: &[(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)],
    right: &[(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)],
) -> Result<(), Error> {
    assert_eq!(left.len(), right.len());
    for (i, ((lname, largs), (rname, rargs))) in left.iter().zip(right.iter()).enumerate() {
        let cfg = self.config();
        let alpha = cfg.alpha;

        layouter.assign_region(
            || format!("cmp pair {}", i),
            |mut region| {
                // fold LHS
                let mut acc_l = Value::known(Fp::zero());
                for (row, inp) in std::iter::once(lname).chain(largs.iter()).enumerate() {
                    region.assign_fixed(|| "q", cfg.q, row, || Value::known(Fp::one()))?;
                    region.assign_advice(|| "token_L", cfg.token, row, || inp.value().copied())?;
                    region.assign_advice(|| "acc_L", cfg.acc, row, || acc_l)?;
                    acc_l = acc_l.zip(inp.value()).map(|(a, t)| a * alpha + t);
                }
                let l_combined = region.assign_advice(|| "L_final", cfg.acc, largs.len() + 1, || acc_l)?;

                // fold RHS
                let mut acc_r = Value::known(Fp::zero());
                for (row, inp) in std::iter::once(rname).chain(rargs.iter()).enumerate() {
                    region.assign_fixed(|| "q", cfg.q, row + 20, || Value::known(Fp::one()))?;
                    region.assign_advice(|| "token_R", cfg.token, row + 20, || inp.value().copied())?;
                    region.assign_advice(|| "acc_R", cfg.acc, row + 20, || acc_r)?;
                    acc_r = acc_r.zip(inp.value()).map(|(a, t)| a * alpha + t);
                }
                let r_combined = region.assign_advice(|| "R_final", cfg.acc, rargs.len() + 21, || acc_r)?;

                // equality constraint (ugyanabban a régióban!)
                region.constrain_equal(l_combined.cell(), r_combined.cell())
            },
        )?;
    }
    Ok(())
}









fn fold_one_term_as_rlc(
    layouter: &mut impl Layouter<Fp>,
    rlc_chip: &RlcFixedChip,
    term: &TermFp,
) -> Result<AssignedCell<Fp, Fp>, Error> {
    // 1) tokenek: name + args (MAX_ARITY-re padelt args a TermFp-ben)
    let mut tokens: Vec<Value<Fp>> = Vec::with_capacity(1 + MAX_ARITY);
    tokens.push(Value::known(term.name));
    for i in 0..MAX_ARITY {
        tokens.push(Value::known(term.args.get(i).copied().unwrap_or(Fp::zero())));
    }
    // 2) RLC assign → combined
    let (combined, _token_cells) = rlc_chip.assign(
        layouter.namespace(|| "RLC(term)"),
        &tokens,
    )?;
    Ok(combined)
}


pub fn cmp_term_lists_pairwise_with_rlc(
    mut layouter: impl Layouter<Fp>,
    rlc_chip: &RlcFixedChip,
    left: &[TermFp],
    right: &[TermFp],
) -> Result<(), Error> {
    // 1) padelés MAX_PAIRS-re
    let empty = TermFp { name: Fp::zero(), args: vec![Fp::zero(); MAX_ARITY] };

    let mut lpad: Vec<TermFp> = left.to_vec();
    let mut rpad: Vec<TermFp> = right.to_vec();
    lpad.resize(MAX_PAIRS, empty.clone());
    rpad.resize(MAX_PAIRS, empty);

    // 2) mindkét oldalt felgörgetjük
    let mut l_combined = Vec::with_capacity(MAX_PAIRS);
    let mut r_combined = Vec::with_capacity(MAX_PAIRS);

    for i in 0..MAX_PAIRS {
        let lc = Self::fold_one_term_as_rlc(&mut layouter, rlc_chip, &lpad[i])?;
        let rc = Self::fold_one_term_as_rlc(&mut layouter, rlc_chip, &rpad[i])?;
        l_combined.push(lc);
        r_combined.push(rc);
    }

    // 3) páronként equality
    layouter.assign_region(
        || "pairwise body==subtree",
        |mut region| {
            for i in 0..MAX_PAIRS {
                region.constrain_equal(l_combined[i].cell(), r_combined[i].cell())?;
            }
            Ok(())
        },
    )?;

    Ok(())
}


}



