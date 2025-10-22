use std::sync::Mutex;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
};
use crate::{
    chips::{
         poseidon_hash::{HashEqChip, HashEqConfig}, rlc_chip::RlcFixedChip, rlc_goal_check_chip::{RlcGoalCheckChip, RlcGoalCheckConfig}, ConsistencyChip, DotChip
    },
    data::{RuleTemplateFileFp, TermFp, UnificationInputFp},
    utils_2::common_helpers::{to_fp_value, MAX_ARITY, MAX_PAIRS},
};
use once_cell::sync::Lazy;

pub const MAX_DOT_DIM: usize = 10;
// Global constraint counter
pub static TOTAL_CONSTRAINTS: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

// Adds to the global constraint count
pub fn add_constraints(n: u64) {
    let mut counter = TOTAL_CONSTRAINTS.lock().unwrap();
    *counter += n;
}

// Reads the current total constraint count
pub fn get_constraints() -> u64 {
    let counter = TOTAL_CONSTRAINTS.lock().unwrap();
    *counter
}

#[derive(Debug, Clone)]
pub struct UnifCompareConfig {
    pub term_name: Column<Advice>,
    pub term_args: [Column<Advice>; MAX_ARITY],
}

impl UnifCompareConfig {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> Self {
        let term_name = meta.advice_column();
        let term_args = array_init::array_init(|_| meta.advice_column());
        meta.enable_equality(term_name);
        for c in term_args.iter() {
            meta.enable_equality(*c);
        }
        Self { term_name, term_args }
    }
}


// Circuit definition
#[derive(Debug, Clone)]
pub struct UnificationCircuit {
    pub rules: RuleTemplateFileFp,
    pub unif: UnificationInputFp,

}

#[derive(Clone, Debug)]
pub struct UnifConfig {
    pub cons_cfg: <ConsistencyChip as Chip<Fp>>::Config,
    pub dot_cfg: <DotChip as Chip<Fp>>::Config,
    pub hash_cfg: HashEqConfig,
    pub rlc_cfg: <RlcFixedChip as Chip<Fp>>::Config,
    pub goal_check_cfg: RlcGoalCheckConfig,
    pub unif_cmp_cfg: UnifCompareConfig,
}

impl Circuit<Fp> for UnificationCircuit {
    type Config = UnifConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            rules: RuleTemplateFileFp {
                predicates: vec![],
                facts: vec![],
            },
            unif: UnificationInputFp {
                goal_name: TermFp::default(),
                goal_term_args: vec![],
                goal_term_name: Fp::zero(),
                unif_body: vec![],
                unif_goal: TermFp::default(),
                substitution: vec![],
                subtree_goals: vec![],
            },
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let alpha = {
            // pl. használd a már meglévő to_fp_value()-t
            to_fp_value("rlc_alpha_v1")
        };
        let cons_cfg = ConsistencyChip::configure(meta);
        let dot_cfg = DotChip::configure(meta);
        let hash_cfg = HashEqChip::configure(meta);
        let goal_check_cfg = RlcGoalCheckChip::configure(meta, alpha);
        
        let rlc_cfg = RlcFixedChip::configure(meta, alpha);
        let unif_cmp_cfg = UnifCompareConfig::configure(meta);
        //let instance = meta.instance_column();
        //meta.enable_equality(instance);


        UnifConfig { cons_cfg, dot_cfg, hash_cfg, rlc_cfg, goal_check_cfg,unif_cmp_cfg,/*, instance*/ }
    }

    fn synthesize(
        &self,
        cfg: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        use halo2_proofs::circuit::Value;

        /*// ✅ Flatten the rules into Fp values
        let flat = crate::utils_2::common_helpers::flatten_rule_template_to_fp(&self.rules);
        let leaves: Vec<Value<Fp>> = flat.iter().map(|&x| Value::known(x)).collect();

        // ✅ Expected hash from public instance
        let expected_cell = layouter.assign_region(
            || "load expected hash (instance)",
            |mut region| {
                region.assign_advice_from_instance(
                    || "expected hash",
                    cfg.instance,
                    0, // public input row = 0
                    cfg.hash_cfg.expected_col,
                    0,
                )
            },
        )?;

        // ✅ Compute Poseidon tree hash for all flattened Fp values
        let root_cell = HashEqChip::tree_hash_all(
            &cfg.hash_cfg,
            layouter.namespace(|| "rulehash full tree"),
            &leaves,
        )?;

        // ✅ Enforce equality: hash(root) == expected (public input)
        layouter.assign_region(
            || "enforce tree root == expected",
            |mut region| {
                region.constrain_equal(root_cell.cell(), expected_cell.cell())?;
                Ok(())
            },
        )?;*/

        // ✅ Continue with other checks

        let (
        goal_name_cell,
        goal_name_arg_cells,
        unif_goal_name_cell,
        unif_goal_arg_cells,
        term_name_cell,
        term_arg_cells,
    ) = layouter.assign_region(
        || "Bind parent inputs",
        |mut region| {
            // --- 1️⃣ Goal name + args ---
            let goal_name_cell = region.assign_advice(
                || "goal_name_from_parent",
                cfg.goal_check_cfg.goal_name,
                0,
                || Value::known(self.unif.goal_name.name),
            )?;

            let mut goal_name_arg_cells = Vec::new();
            for (i, arg) in self.unif.goal_name.args.iter().enumerate() {
                let cell = region.assign_advice(
                    || format!("goal_name_arg_{}", i),
                    cfg.goal_check_cfg.goal_name, // akár külön column is lehet
                    1 + i,
                    || Value::known(*arg),
                )?;
                goal_name_arg_cells.push(cell);
            }

            // --- 2️⃣ Unification goal name + args ---
            // az unif_goal is TermFp, tehát ugyanúgy kezeljük
            let base_row = 1 + self.unif.goal_name.args.len() + 1; // eltoljuk a sorindexet
            let unif_goal_name_cell = region.assign_advice(
                || "unif_goal_name_from_parent",
                cfg.goal_check_cfg.unif_goal,
                base_row,
                || Value::known(self.unif.unif_goal.name),
            )?;

            let mut unif_goal_arg_cells: Vec<halo2_proofs::circuit::AssignedCell<Fp, Fp>> = Vec::new();
            for (i, arg) in self.unif.unif_goal.args.iter().enumerate() {
                let cell = region.assign_advice(
                    || format!("unif_goal_arg_{}", i),
                    cfg.goal_check_cfg.unif_goal,
                    base_row + 1 + i,
                    || Value::known(*arg),
                )?;
                unif_goal_arg_cells.push(cell);
            }

            // --- 3️⃣ Goal term name + args (ahogy eddig volt) ---
            let next_row = base_row + 1 + self.unif.unif_goal.args.len() + 1;
            let term_name_cell = region.assign_advice(
                || "goal_term_name",
                cfg.goal_check_cfg.goal_name,
                next_row,
                || Value::known(self.unif.goal_term_name),
            )?;

            let mut term_arg_cells = Vec::new();
            for (i, arg) in self.unif.goal_term_args.iter().enumerate() {
                let cell = region.assign_advice(
                    || format!("goal_term_arg_{}", i),
                    cfg.goal_check_cfg.goal_name, // akár külön column is lehet
                    next_row + 1 + i,
                    || Value::known(*arg),
                )?;
                term_arg_cells.push(cell);
            }

            Ok((
                goal_name_cell,
                goal_name_arg_cells,
                unif_goal_name_cell,
                unif_goal_arg_cells,
                term_name_cell,
                term_arg_cells,
            ))
        },
    )?;



    let goal_check = RlcGoalCheckChip::construct(cfg.goal_check_cfg.clone());
    let _combined_cell = goal_check.assign(
        layouter.namespace(|| "GoalCheck"),
        &goal_name_cell,
        &goal_name_arg_cells,
        &term_name_cell,
        &term_arg_cells,
        &unif_goal_name_cell,
        &unif_goal_arg_cells,
    )?;



let (body_pairs, subtree_pairs) = bind_body_and_subtree_as_cells_padded(
        "Bind body & subtree (padded)",
        &mut layouter,
        &cfg.unif_cmp_cfg,                 // term_name + term_args oszlopok
        &self.unif.unif_body,              // Vec<TermFp>
        &self.unif.subtree_goals,          // Vec<TermFp>
    )?;  // -> Vec<(name_cell, Vec<arg_cells>)> mindkét oldalra, hossza MAX_PAIRS

    // === 3) RLC-vel páronkénti equality (teljes huzalozás!) ===
    let rlc_chip = RlcFixedChip::construct(cfg.rlc_cfg.clone());

    rlc_chip.cmp_term_lists_pairwise_with_rlc_cells(
        layouter.namespace(|| "Compare body vs subtree (wired RLC)"),
        &body_pairs,
        &subtree_pairs,
    )?;


        /*if self.unif.subtree_goals.is_empty() {
            println!("Total constraints so far: {} (fact)", get_constraints());
            return Ok(());
        }

        let (matching_clause, structure_vec, witness_vec) =
            get_matching_structure_and_vectors(&self.unif, &self.rules)?;

        add_constraints(1);
        cons_chip.assign_pairs2(
            layouter.namespace(|| format!("pred_clause_dotcheck")),
            &[(structure_vec, witness_vec)],
        )?;

        let (padded_w_vec, padded_compressed_rows) = get_w_and_v_vec(
            &matching_clause,
            &self.unif.goal_term_args,
            &self.unif.unif_body,
            "seed",
            MAX_DOT_DIM,
        )?;

        add_constraints((MAX_DOT_DIM * 2 + 1) as u64);
        dot_chip.assign_dot_check(
            layouter.namespace(|| format!("variable_dot_check:")),
            &padded_w_vec,
            &padded_compressed_rows,
            Fp::zero(),
        )?;

        println!("Total constraints so far: {} (predicate)", get_constraints());*/
        Ok(())
    }
}


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