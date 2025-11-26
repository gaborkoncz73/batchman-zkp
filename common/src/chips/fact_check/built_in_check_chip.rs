use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error},
};

use halo2curves::ff::Field;
use crate::utils_2::common_helpers::to_fp_value;

// ─────────────────────────────────────────────────────────────────────────────
// Konfig
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Clone, Debug)]
pub struct BuiltinExprConfig {
    /// Egyetlen "work" advice oszlop, ebben számolunk mindent (egyszerű, robusztus)
    pub work: Column<Advice>,
}

#[derive(Clone, Debug)]
pub struct BuiltinExprChip {
    cfg: BuiltinExprConfig,
}

impl Chip<Fp> for BuiltinExprChip {
    type Config = BuiltinExprConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config { &self.cfg }
    fn loaded(&self) -> &Self::Loaded { &() }
}

impl BuiltinExprChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> BuiltinExprConfig {
        let work = meta.advice_column();
        meta.enable_equality(work);
        BuiltinExprConfig { work }
    }

    pub fn construct(cfg: BuiltinExprConfig) -> Self {
        Self { cfg }
    }

    #[inline]
    fn fp_op(s: &str) -> Fp { to_fp_value(s) }

    /// b = [x == c] tanú + booleanitás
    fn eq_const_bool(
        &self,
        mut layouter: impl Layouter<Fp>,
        x: &AssignedCell<Fp,Fp>,
        c: Fp,
        row0: usize,
    ) -> Result<AssignedCell<Fp,Fp>, Error> {
        let col = self.cfg.work;
        layouter.assign_region(
            || "eq_const_bool",
            |mut region| {
                let b = region.assign_advice(
                    || "b",
                    col, row0,
                    || x.value().map(|vx| if *vx == c { Fp::ONE } else { Fp::ZERO })
                )?;
                let be = region.assign_advice(
                    || "b*(1-b)",
                    col, row0+1,
                    || b.value().map(|vb| *vb * (Fp::ONE - *vb))
                )?;
                let z = region.assign_advice(|| "z", col, row0+2, || Value::known(Fp::ZERO))?;
                //region.constrain_equal(be.cell(), z.cell())?;
                Ok(b)
            }
        )
    }

    /// s_add + s_sub + s_mul + s_div = 1  és mind boolean
    fn enforce_onehot4(
        &self,
        mut layouter: impl Layouter<Fp>,
        s: [&AssignedCell<Fp,Fp>;4],
        row: usize,
    ) -> Result<(), Error> {
        let col = self.cfg.work;
        layouter.assign_region(
            || "onehot4",
            |mut region| {
                for (i,si) in s.iter().enumerate() {
                    let be = region.assign_advice(
                        || format!("b*(1-b)[{i}]"),
                        col, row+i,
                        || si.value().map(|v| *v * (Fp::ONE - *v))
                    )?;
                    let z = region.assign_advice(|| "z", col, row+10+i, || Value::known(Fp::ZERO))?;
                    region.constrain_equal(be.cell(), z.cell())?;
                }
                let sum = region.assign_advice(
                    || "sum selectors",
                    col, row+20,
                    || s[0].value()
                        .zip(s[1].value()).zip(s[2].value()).zip(s[3].value())
                        .map(|(((a,b),c),d)| *a + *b + *c + *d)
                )?;
                let diff = region.assign_advice(
                    || "sum-1",
                    col, row+21,
                    || sum.value().map(|v| *v - Fp::ONE)
                )?;
                let z = region.assign_advice(|| "z", col, row+22, || Value::known(Fp::ZERO))?;
                region.constrain_equal(diff.cell(), z.cell())
            }
        )
    }

    /// Fő API: kiértékeli a láncot és visszaad egy boolean `ok` cellát.
    ///
    /// Bemenet:
    ///  - `names[p]`  : a predikátumok/builtinek nevei; p=0: "is" vagy "="; p>0: +,-,*,div
    ///  - `args[p][a][l]`: argument mátrixok (operand mindig args[p][0][0]),
    ///     * p=0-nál: LHS=args[0][0][0], RHS0=args[0][1][0]
    ///
    /// Viselkedés:
    ///  - Ellenőrzi, hogy p=0 neve is/=
    ///  - Akkumulátor: acc = RHS0
    ///  - p>=1: unáris műveletek acc-ra operand=arg0[0]
    ///  - Végül ok = [LHS == acc] * [name0 ∈ {is,=}]
    pub fn eval_chain_equal(
        &self,
        mut layouter: impl Layouter<Fp>,
        names: &[AssignedCell<Fp,Fp>],
        args: &[Vec<Vec<AssignedCell<Fp,Fp>>>],
        must_be_true: bool,
    ) -> Result<AssignedCell<Fp,Fp>, Error> {
        assert!(!names.is_empty(), "names must be non-empty");
        let col = self.cfg.work;

        // 0. term: is / =
        let name0 = &names[0];
        let lhs   = args[0][0][0].clone();
        let rhs0  = args[0][1][0].clone();

        // comparison path számára KÜLÖN clone-ok
        let lhs_cmp = lhs.clone();
        let rhs_cmp = rhs0.clone();
        // név-egyezés booleánok (is vagy =)
        let b_is = self.eq_const_bool(layouter.namespace(|| "name==is"), name0, Self::fp_op("is"), 0)?;
        let b_eq = self.eq_const_bool(layouter.namespace(|| "name==="),  name0, Self::fp_op("="),  5)?;
        let b_gt = self.eq_const_bool(layouter.namespace(|| "name >"), name0, Self::fp_op(">"), 10)?;
        let b_lt = self.eq_const_bool(layouter.namespace(|| "name <"), name0, Self::fp_op("<"), 20)?;
        let b_ge = self.eq_const_bool(layouter.namespace(|| "name >="), name0, Self::fp_op(">="), 30)?;
        let b_le = self.eq_const_bool(layouter.namespace(|| "name <="), name0, Self::fp_op("<="), 40)?;


        // legalább az egyik igaz: b_ie = 1 - (1-b_is)*(1-b_eq)
        // legalább az egyik igaz: b_ie = 1 - (1-b_is)*(1-b_eq)  (EZZEL TÉRJÜNK VISSZA AZ EREDETI EQ-ÁGHOZ)
        let b_ie = layouter.assign_region(
            || "is_or_eq = 1 - (1-bis)*(1=)",
            |mut region| {
                let col = self.cfg.work;
                let prod_not = region.assign_advice(
                    || "(1-bis)*(1=)",
                    col, 20,
                    || b_is.value().zip(b_eq.value()).map(|(bi,be)| (Fp::ONE - *bi) * (Fp::ONE - *be))
                )?;
                region.assign_advice(
                    || "is_or_eq",
                    col, 21,
                    || prod_not.value().map(|v| Fp::ONE - *v)
                )
            }
        )?;

        // ÚJ: b_cmp = OR(>, <, >=, <=) = 1 - Π(1 - b_op)
        let b_cmp = layouter.assign_region(
            || "cmp_ops_or",
            |mut region| {
                let col = self.cfg.work;
                let prod_not = region.assign_advice(
                    || "Π(1-bop)",
                    col, 30,
                    || b_gt.value()
                        .zip(b_lt.value())
                        .zip(b_ge.value())
                        .zip(b_le.value())
                        .map(|(((gt,lt),ge),le)| {
                            let one = Fp::ONE;
                            (one - *gt) * (one - *lt) * (one - *ge) * (one - *le)
                        })
                )?;
                region.assign_advice(
                    || "b_cmp",
                    col, 31,
                    || prod_not.value().map(|v| Fp::ONE - *v)
                )
            }
        )?;

        // ONE-HOT(2): b_ie + b_cmp == 1  (pont egy ág legyen aktív)
        let b_known = layouter.assign_region(
            || "known op",
            |mut region| {
                let col = self.cfg.work;
                let prod_not = region.assign_advice(
                    || "(1-b_ie)*(1-b_cmp)",
                    col, 50,
                    || b_ie.value().zip(b_cmp.value()).map(|(ie,cmp)| {
                        let one = Fp::ONE;
                        (one - *ie) * (one - *cmp)
                    })
                )?;
                region.assign_advice(
                    || "b_known",
                    col, 51,
                    || prod_not.value().map(|v| Fp::ONE - v)
                )
            }
        )?;

        // ✅ csak akkor kényszerítjük az egy-szor-aktiv-at, ha ismert ág
        layouter.assign_region(
            || "gate onehot2",
            |mut region| {
                let col = self.cfg.work;
                let sum = region.assign_advice(
                    || "sum",
                    col, 52,
                    || b_ie.value().zip(b_cmp.value()).map(|(ie,cmp)| *ie + *cmp)
                )?;
                let diff = region.assign_advice(
                    || "b_known*(sum-1)",
                    col, 53,
                    || b_known.value().zip(sum.value()).map(|(k,s)| *k * (*s - Fp::ONE))
                )?;
                let z = region.assign_advice(|| "z", col, 54, || Value::known(Fp::ZERO))?;
                region.constrain_equal(diff.cell(), z.cell())
            }
        )?;

        // induló acc
        let mut acc = rhs0;

        // p>=1: unáris műveletek
        for (p, name_cell) in names.iter().enumerate().skip(1) {
            let x = args[p][0][0].clone();

            // szelektor booleánok
            let s_add = self.eq_const_bool(layouter.namespace(|| format!("p{p} name==+")),   name_cell, Self::fp_op("+"),   100 + 40*p)?;
            let s_sub = self.eq_const_bool(layouter.namespace(|| format!("p{p} name==-")),  name_cell, Self::fp_op("-"),   110 + 40*p)?;
            let s_mul = self.eq_const_bool(layouter.namespace(|| format!("p{p} name==*")),  name_cell, Self::fp_op("*"),   120 + 40*p)?;
            let s_div = self.eq_const_bool(layouter.namespace(|| format!("p{p} name==div")),name_cell, Self::fp_op("div"), 130 + 40*p)?;

            // arithmetic selector = OR of add/sub/mul/div
            let s_arith = layouter.assign_region(
                || format!("s_arith p{p}"),
                |mut region| {
                    let col = self.cfg.work;
                    region.assign_advice(
                        || "s_add + s_sub + s_mul + s_div",
                        col, 140 + 40*p,
                        || s_add.value()
                            .zip(s_sub.value()).zip(s_mul.value()).zip(s_div.value())
                            .map(|(((a,b),c),d)| *a + *b + *c + *d)
                    )
                }
            )?;

            // soft check: s_arith*(1 - s_arith) == 0  (boolean)
            layouter.assign_region(
                || format!("s_arith boolean p{p}"),
                |mut region| {
                    let col = self.cfg.work;
                    let be = region.assign_advice(
                        || "s_arith*(1-s_arith)",
                        col, 141 + 40*p,
                        || s_arith.value().map(|v| *v * (Fp::ONE - *v))
                    )?;
                    let z = region.assign_advice(|| "z", col, 142 + 40*p, || Value::known(Fp::ZERO))?;
                    region.constrain_equal(be.cell(), z.cell())
                }
            )?;


            // res-ek
            let res_add = layouter.assign_region(
                || format!("res_add p{p}"),
                |mut region| {
                    region.assign_advice(
                        || "acc+x",
                        col, 200 + 10*p,
                        || acc.value().zip(x.value()).map(|(a,b)| *a + *b)
                    )
                }
            )?;
            let res_sub = layouter.assign_region(
                || format!("res_sub p{p}"),
                |mut region| {
                    region.assign_advice(
                        || "acc-x",
                        col, 201 + 10*p,
                        || acc.value().zip(x.value()).map(|(a,b)| *a - *b)
                    )
                }
            )?;
            let res_mul = layouter.assign_region(
                || format!("res_mul p{p}"),
                |mut region| {
                    region.assign_advice(
                        || "acc*x",
                        col, 202 + 10*p,
                        || acc.value().zip(x.value()).map(|(a,b)| *a * *b)
                    )
                }
            )?;
            // div: tanú inv(x), és kapu s_div*(x*inv - 1) == 0
            // div: egész osztás q = floor(acc / x), r = acc - q*x, 0 <= r < x
            // div: integer division acc = q*x + r, 0 <= r < x
            let (res_div, _q, _r) = layouter.assign_region(
            || format!("res_div integer p{p}"),
            |mut region| {
                // q = floor(acc/x)
                let q_val = acc.value().zip(x.value()).map(|(a, xv)| {
                    if *xv == Fp::ZERO {
                        // osztás 0-val: adunk q=0-t; (ha akarod, külön tiltó kaput is tehetsz s_div & (x==0) == 0)
                        Fp::ZERO
                    } else {
                        let a128 = fp_to_u128(*a);
                        let x128 = fp_to_u128(*xv);
                        Fp::from((a128 / x128) as u64)
                    }
                });
                let q = region.assign_advice(|| "q", col, 203 + 10*p, || q_val)?;

                // r = acc - q*x
                let r = region.assign_advice(
                    || "r",
                    col, 204 + 10*p,
                    || acc.value().zip(q.value()).zip(x.value()).map(|((a,qv), xv)| {
                        *a - (*qv * *xv)
                    })
                )?;

                // acc = q*x + r  ✔
                // ✅ csak ha s_div=1, akkor kötelező: acc = q*x + r
                let gated = region.assign_advice(
                    || "s_div * (acc - q*x - r)",
                    col, 205 + 10*p,
                    || s_div.value()
                        .zip(acc.value()).zip(q.value()).zip(x.value()).zip(r.value())
                        .map(|((((sd,a),qv),xv),rv)| *sd * (*a - (*qv * *xv) - *rv))
                )?;
                let zero = region.assign_advice(|| "0", col, 206 + 10*p, || Value::known(Fp::ZERO))?;
                region.constrain_equal(gated.cell(), zero.cell())?;


                // r < x  → opcionális “soft” check (range tábla nélkül ez csak rekonstrukció)
                let _y = region.assign_advice(
                    || "y = x-r-1",
                    col, 207 + 10*p,
                    || x.value().zip(r.value()).map(|(xv,rv)| *xv - *rv - Fp::ONE)
                )?;

                // (opcionális) "range_check_u32_in_region" hívások maradhatnak, de valódi 16 bites bound-ot
                // nem kényszerítenek lookup tábla nélkül. Ha kell erős bound, külön range-chip kell.

                // integer div eredménye → accumulator kandidát
                let rd = region.assign_advice(
                    || "acc=q",
                    col, 270 + 20*p,
                    || q.value().copied()
                )?;

                Ok((rd, q, r))
            }
            )?;

            // mux: acc_next = Σ s_i * res_i
            acc = layouter.assign_region(
            || format!("acc_next p{p}"),
            |mut region| {
                region.assign_advice(
                    || "acc + Σ s_i*(res_i-acc)",
                    col, 207 + 10*p,
                    || s_add.value().zip(res_add.value())
                        .zip(s_sub.value()).zip(res_sub.value())
                        .zip(s_mul.value()).zip(res_mul.value())
                        .zip(s_div.value()).zip(res_div.value())
                        .zip(acc.value())
                        .map(|((((((((sa,ra), ss), rs), sm), rm), sd), rd), accv)| {
                            let term = *sa * (*ra - *accv)
                                    + *ss * (*rs - *accv)
                                    + *sm * (*rm - *accv)
                                    + *sd * (*rd - *accv);
                            *accv + term
                        })
                )
            }
        )?;
        }

        // végső összehasonlítás + AND az is/=-szel
        //---------------------------------------------
        // EDDIGI eq művelet eredménye acc-ban van

        // delta = lhs - rhs0
        let delta = layouter.assign_region(
            || "delta",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "lhs-rhs",
                    col, 700,
                    || lhs_cmp.value().zip(rhs_cmp.value()).map(|(l,r)| *l - *r)
                )
            }
        )?;

        // (delta > 0) gated by b_cmp
        let is_positive = layouter.assign_region(
            || "is_positive_gated",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "pos(d)*b_cmp",
                    col, 701,
                    || delta.value().zip(b_cmp.value())
                        .map(|(d,bc)| if *bc == Fp::ZERO { Fp::ZERO }
                                    else if *d > Fp::ZERO { Fp::ONE } else { Fp::ZERO })
                )
            }
        )?;


        // Soft: delta < 0
        let is_negative = layouter.assign_region(
            || "is_negative",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "neg(d)",
                    col, 702,
                    || delta.value().map(|d| if *d < Fp::ZERO { Fp::ONE } else { Fp::ZERO })
                )
            }
        )?;

        // d == 0 → eq result
        let is_zero = layouter.assign_region(
            || "is_zero(d)",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "zero(d)",
                    col, 703,
                    || delta.value().map(|d| if *d == Fp::ZERO { Fp::ONE } else { Fp::ZERO })
                )
            }
        )?;

        // Comparison op logic
        let ok_gt = layouter.assign_region(
            || "ok_gt",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "",
                    col, 704,
                    || is_positive.value().zip(b_gt.value()).map(|(p,g)| *p * *g)
                )
            }
        )?;
        let ok_lt = layouter.assign_region(
            || "ok_lt",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "",
                    col, 705,
                    || is_negative.value().zip(b_lt.value()).map(|(n,l)| *n * *l)
                )
            }
        )?;
        let ok_ge = layouter.assign_region(
            || "ok_ge",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "",
                    col, 706,
                    || is_positive.value().zip(is_zero.value()).zip(b_ge.value())
                        .map(|((p,z),ge)| (*p + *z) * *ge)
                )
            }
        )?;
        let ok_le = layouter.assign_region(
            || "ok_le",
            |mut region| {
                let col = self.cfg.work;
                region.assign_advice(
                    || "",
                    col, 707,
                    || is_negative.value().zip(is_zero.value()).zip(b_le.value())
                        .map(|((n,z),le)| (*n + *z) * *le)
                )
            }
        )?;

// OR of all comparison results
let cmp_ok = layouter.assign_region(
    || "cmp_ok",
    |mut region| {
        let col = self.cfg.work;
        region.assign_advice(
            || "",
            col, 708,
            || ok_gt.value()
                 .zip(ok_lt.value())
                 .zip(ok_ge.value())
                 .zip(ok_le.value())
                 .map(|(((a,b),c),d)| {
                     let one = Fp::ONE;
                     one - ((one-*a)*(one-*b)*(one-*c)*(one-*d))
                 })
        )
    }
)?;

// eq_ok: [lhs == acc]
let eq_ok = layouter.assign_region(
    || "[lhs==acc]",
    |mut region| {
        let col = self.cfg.work;
        region.assign_advice(
            || "eq_ok",
            col, 709,
            || lhs.value().zip(acc.value()).map(|(l,a)| if *l == *a { Fp::ONE } else { Fp::ZERO })
        )
    }
)?;

// final ok = b_ie*eq_ok + b_cmp*cmp_ok
let ok = layouter.assign_region(
    || "final ok (mux eq vs cmp)",
    |mut region| {
        let col = self.cfg.work;
        region.assign_advice(
            || "ok",
            col, 710,
            || b_ie.value().zip(eq_ok.value()).zip(b_cmp.value()).zip(cmp_ok.value())
                .map(|(((ie,eqv),cmp),cmpv)| *ie * *eqv + *cmp * *cmpv)
        )
    }
)?;

// booleanitás: ok*(1-ok) == 0
layouter.assign_region(
    || "ok boolean",
    |mut region| {
        let col = self.cfg.work;
        let be = region.assign_advice(
            || "ok*(1-ok)",
            col, 711,
            || ok.value().map(|v| *v * (Fp::ONE - *v))
        )?;
        let z = region.assign_advice(|| "z", col, 712, || Value::known(Fp::ZERO))?;
        region.constrain_equal(be.cell(), z.cell())
    }
)?;


        // ha kell: ok == 1 kényszer
        if must_be_true {
            layouter.assign_region(
                || "ok==1",
                |mut region| {
                    let diff = region.assign_advice(
                        || "ok-1",
                        col, 904,
                        || ok.value().map(|v| *v - Fp::ONE)
                    )?;
                    let z = region.assign_advice(|| "z", col, 905, || Value::known(Fp::ZERO))?;
                    region.constrain_equal(diff.cell(), z.cell())
                }
            )?;
        }

        Ok(ok)
    }
}
use halo2curves::ff::PrimeField;

// extract lower 64-bit integer safely from field
fn fp_to_u128(x: Fp) -> u128 {
    let repr = x.to_repr();
    let mut b = [0u8; 16];
    b.copy_from_slice(&repr.as_ref()[0..16]);
    u128::from_le_bytes(b)
}

/// Soft 32-bit range-check (u32)
fn range_check_u32_in_region(
    region: &mut halo2_proofs::circuit::Region<'_, Fp>,
    col: Column<Advice>,
    row: usize,
    v: &AssignedCell<Fp, Fp>,
) -> Result<(), Error> {
    let lo = region.assign_advice(
        || "lo",
        col, row,
        || v.value().map(|val| {
            let vv = fp_to_u128(*val);
            Fp::from((vv & 0xFFFF) as u64)
        })
    )?;
    let hi = region.assign_advice(
        || "hi",
        col, row+1,
        || v.value().map(|val| {
            let vv = fp_to_u128(*val);
            Fp::from(((vv >> 16) & 0xFFFF) as u64)
        })
    )?;

    // v == lo + hi * 2^16
    let recon = region.assign_advice(
        || "recon",
        col, row+2,
        || lo.value().zip(hi.value()).map(|(l,h)| {
            *l + (*h * Fp::from(1u64 << 16))
        })
    )?;

    region.constrain_equal(v.cell(), recon.cell())?;

    Ok(())
}
