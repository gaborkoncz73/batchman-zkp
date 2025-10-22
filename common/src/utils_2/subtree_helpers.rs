use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    pasta::Fp,
    plonk::Error};

use crate::{data::TermFp, unification_checker_circuit::UnifCompareConfig, utils_2::common_helpers::{MAX_ARITY, MAX_PAIRS}};

fn bind_body_and_subtree_as_cells_padded(
    region_name: &str,
    layouter: &mut impl Layouter<Fp>,
    cfg: &UnifCompareConfig,           // term_name + term_args oszlopok
    body: &[TermFp],
    subtree: &[TermFp],
) -> Result<
    (
        Vec<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)>, // body_pairs
        Vec<(AssignedCell<Fp, Fp>, Vec<AssignedCell<Fp, Fp>>)>, // subtree_pairs
    ),
    Error,
> {
    use halo2_proofs::circuit::Value;

    let stride = 1 + MAX_ARITY; // soronként: 1 név + MAX_ARITY arg
    layouter.assign_region(
        || region_name,
        |mut region| {
            let mut body_pairs   = Vec::with_capacity(MAX_PAIRS);
            let mut subtree_pairs= Vec::with_capacity(MAX_PAIRS);

            // body blokkok 0..MAX_PAIRS
            for i in 0..MAX_PAIRS {
                let row0 = i * stride;
                let term = body.get(i);

                // name
                let name_val = Value::known(term.map(|t| t.name).unwrap_or(Fp::zero()));
                let name_cell = region.assign_advice(
                    || format!("body[{i}].name"),
                    cfg.term_name,
                    row0,
                    || name_val,
                )?;

                // args
                let mut args_cells = Vec::with_capacity(MAX_ARITY);
                for j in 0..MAX_ARITY {
                    let aval = Value::known(
                        term.and_then(|t| t.args.get(j).copied()).unwrap_or(Fp::zero())
                    );
                    let c = region.assign_advice(
                        || format!("body[{i}].arg{j}"),
                        cfg.term_args[j],
                        row0 + 1 + j,
                        || aval,
                    )?;
                    args_cells.push(c);
                }
                body_pairs.push((name_cell, args_cells));
            }

            // subtree blokkok: eltolással, hogy ne fedjék egymást
            let base = MAX_PAIRS * stride + 8; // kis puffer
            for i in 0..MAX_PAIRS {
                let row0 = base + i * stride;
                let term = subtree.get(i);

                // name
                let name_val = Value::known(term.map(|t| t.name).unwrap_or(Fp::zero()));
                let name_cell = region.assign_advice(
                    || format!("subtree[{i}].name"),
                    cfg.term_name,
                    row0,
                    || name_val,
                )?;

                // args
                let mut args_cells = Vec::with_capacity(MAX_ARITY);
                for j in 0..MAX_ARITY {
                    let aval = Value::known(
                        term.and_then(|t| t.args.get(j).copied()).unwrap_or(Fp::zero())
                    );
                    let c = region.assign_advice(
                        || format!("subtree[{i}].arg{j}"),
                        cfg.term_args[j],
                        row0 + 1 + j,
                        || aval,
                    )?;
                    args_cells.push(c);
                }
                subtree_pairs.push((name_cell, args_cells));
            }

            Ok((body_pairs, subtree_pairs))
        },
    )
}