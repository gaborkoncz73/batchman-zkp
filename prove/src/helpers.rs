use std::collections::HashMap;

use common::{data::{FactEntry, GoalEntry, ProofNode, TermFp, UnificationInputFp}, utils_2::common_helpers::{MAX_ARITY, MAX_PRED_LIST, to_fp_value}};
use halo2_proofs::pasta::Fp;

// From the goal and hashmap it creates the Unification input
pub fn unification_input_from_goal_and_facts(
    g: &GoalEntry,
    facts: &HashMap<String, Fp>
) -> UnificationInputFp {

    // ✅ goal is now a Vec<TermFp>
    let goal_name_terms: Vec<TermFp> =
        encode_str_to_termfp(&g.goal, facts);

    // ✅ subtree → Vec<Vec<TermFp>>
    let subtree_terms: Vec<Vec<TermFp>> = g.subtree
        .iter()
        .map(|node| encode_proofnode_to_termfp(node, facts))
        .collect();

    UnificationInputFp {
        goal_name: goal_name_terms,
        subtree_goals: subtree_terms,
    }
}

// Converting a goal name into structured TermFp
pub fn encode_str_to_termfp(input: &str, facts: &HashMap<String, Fp>) -> Vec<TermFp> {
    const OPERATORS: [&str; 8] = [" is ", "=", ">", "<", "*", " div ", "+", "-"];
    let mut out_terms = Vec::new();
    let has_op = OPERATORS.iter().any(|op| input.contains(op));
    let has_list = input.contains('[');

    // 1️⃣ Egyszerű predikátum – se op, se lista
    if !has_op && !has_list {
        out_terms.push(encode_str_to_termfp_og(input, facts));
        return out_terms;
    }

    // 2️⃣ Predikátum LISTÁKKAL, de NINCS operator
    // ✅ Predikátum LISTÁKKAL — korrekt lista szétbontás
if !has_op && has_list {
    let open = input.find('(').unwrap_or(input.len());
    let close = input.rfind(')').unwrap_or(input.len());
    let name_str = input[..open].trim();
    let args_str = &input[open + 1..close];
    // ✅ applySocialSupports special handling

    if name_str == "sumOfMonthlyConsumptions" && args_str.contains("[]"){
        out_terms.push(encode_str_to_termfp_og(input, facts));
        return out_terms;
    }
    if name_str == "applySocialSupports" {
        let mut matrix = vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY];

        let parts = split_top_level_commas(args_str)
            .into_iter()
            .map(|s| s.trim())
            .collect::<Vec<_>>();

        if parts.len() >= 3 {
            // 1️⃣ Input paraméter
            matrix[0][0] = to_fp_value(parts[0]);

            // 2️⃣ Credential lista feldolgozás
            let list_arg = parts[1];
            if list_arg.starts_with('[') && list_arg.ends_with(']') {
                let inner = &list_arg[1..list_arg.len() - 1]; // [] nélkül
                let list_parts = split_top_level_commas(inner);

                if let Some(first) = list_parts.first() {
                    if first.starts_with("(") && first.ends_with(")") {
                        // tuple szétszedése
                        let tuple_fields = parse_tuple_fields_keep_all(first);
                        for (i, f) in tuple_fields.into_iter().take(3).enumerate() {
                            matrix[1][i] = to_fp_value(&f);
                        }
                    } else {
                        matrix[1][0] = to_fp_value(first);
                    }
                }
            }

            // 3️⃣ Result paraméter
            matrix[2][0] = to_fp_value(parts[2]);
        }

        out_terms.push(TermFp {
            name: to_fp_value(name_str),
            args: matrix,
            fact_hashes: Fp::zero(),
        });
        return out_terms;
    }

    if name_str == "monthlyConsumptions" {
        let mut matrix = vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY];

        // belső lista -> csak számok vesszőkkel (szögletes zárójelek nélkül)
        let list_inner = args_str
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string();

        matrix[0][0] = to_fp_value(&list_inner);

        out_terms.push(TermFp {
            name: to_fp_value(name_str),
            args: matrix,
            fact_hashes: Fp::zero(),
        });

        return out_terms;
    }
    let pred_name = name_str.to_string();
    let args_vec: Vec<&str> = split_top_level_commas(args_str);

    let mut matrix: Vec<Vec<Fp>> =
        vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY];

    for (arg_i, arg) in args_vec.iter().enumerate() {
        let arg = arg.trim();
            if name_str == "sumOfMonthlyConsumptions" && arg.starts_with('[') && arg.ends_with(']') {
            let inner = &arg[1..arg.len() - 1]; // strip [ ]
            let elems: Vec<&str> = split_top_level_commas(inner)
                .into_iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if !elems.is_empty() {
                // First element
                matrix[arg_i][0] = to_fp_value(elems[0]);

                if elems.len() > 1 {
                    // Remaining elements → merged into ONE string for Tail
                    let tail = elems[1..].join(",");
                    matrix[arg_i][1] = to_fp_value(&tail);
                }

                // Everything else stays empty
            }

            // ✅ IMPORTANT: NO DEFAULT LIST LOGIC EXECUTION!
            continue;
        }
        // ✅ Ha lista: [ ... ]
        if arg.starts_with('[') && arg.ends_with(']') {
            let inner = &arg[1..arg.len() - 1];  // levágjuk a []-t

            if inner.is_empty() {
                matrix[arg_i][0] = Fp::zero();
                continue;
            }

            let mut idx = 0usize;

            // ✅ listán belül split top-level elemekre
            for elem in split_top_level_commas(inner) {
                let elem = elem.trim();

                // ✅ ha tuple: ('A',B,C)
                if elem.starts_with("(") && elem.ends_with(")") {
                    let tuple_parts = parse_tuple_fields_keep_all(elem);
                    for (t_i, part) in tuple_parts.clone().into_iter().enumerate() {
                        matrix[arg_i][idx + t_i] = to_fp_value(&part);
                    }
                    idx += tuple_parts.len();
                } else {
                    matrix[arg_i][idx] = to_fp_value(elem);
                    idx += 1;
                }
            }
        }

        // ✅ Nem lista → sima argumentum
        else {
            matrix[arg_i][0] = to_fp_value(arg);
        }
    }

    out_terms.push(TermFp {
        name: to_fp_value(&pred_name),
        args: matrix,
        fact_hashes: Fp::zero(),
    });

    return out_terms;
}


    // 3️⃣ OPERATOR kezelése (bármi van benne: lista, tuple, predikátum)
    for op in OPERATORS {
        if let Some(pos) = input.find(op) {
            let lhs = input[..pos].trim();
            let rhs = input[pos + op.len()..].trim();
            let op_name = op.trim();

            let mut preds: Vec<TermFp> = Vec::new();

            // ✅ First predicate: op with LHS and ONLY FIRST RHS ARG
            let mut first_args = vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY];
            first_args[0][0] = to_fp_value(lhs);

            // ✅ normalize `div` so splitter separates it
            let rhs_clean = rhs.replace("div", " div ");

            let rhs_first = rhs_clean
                .split([' ', '-', '+', '*', '=', '>', '<'])
                .filter(|s| !s.is_empty())
                .next()
                .unwrap()
                .to_string();

            first_args[1][0] = to_fp_value(&rhs_first);

            preds.push(TermFp {
                name: to_fp_value(op_name),
                args: first_args,
                fact_hashes: Fp::zero(),
            });

            // ✅ remaining operator predicates
            let mut rest = rhs_clean.clone();

            while let Some((next_op_pos, next_op)) =
                OPERATORS.iter()
                .filter_map(|o| rest.find(o).map(|pos| (pos, *o)))
                .next()
            {
                let new_rhs = rest[next_op_pos + next_op.len()..]
                    .trim()
                    .replace("div", " div ");

                let next_name = next_op.trim();

                let mut new_args = vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY];

                if let Some(first) = new_rhs
                    .split([' ', '-', '+', '*', '=', '>', '<'])
                    .filter(|s| !s.is_empty())
                    .next()
                {
                    new_args[0][0] = to_fp_value(first);
                }

                preds.push(TermFp {
                    name: to_fp_value(next_name),
                    args: new_args,
                    fact_hashes: Fp::zero(),
                });

                rest = new_rhs;
            }

            return preds;
        }
    }


    // fallback (sose kéne lefusson)
    out_terms.push(encode_str_to_termfp_og(input, facts));
    out_terms
}


fn parse_tuple_fields_keep_all(atom_str: &str) -> Vec<String> {
    let s = atom_str.trim();
    if !(s.starts_with('(') && s.ends_with(')')) {
        return vec![s.to_string()];
    }
    let inner = &s[1..s.len() - 1];
    split_top_level_commas(inner)
        .into_iter()
        .map(|p| p.trim().to_string())
        .collect()
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth_paren = 0;
    let mut depth_bracket = 0;
    let mut last = 0;
    let chars: Vec<char> = s.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        match ch {
            '(' => depth_paren += 1,
            ')' => if depth_paren > 0 { depth_paren -= 1 },
            '[' => depth_bracket += 1,
            ']' => if depth_bracket > 0 { depth_bracket -= 1 },
            ',' if depth_paren == 0 && depth_bracket == 0 => {
                out.push(&s[last..i]);
                last = i + 1;
            }
            _ => {}
        }
    }

    if last <= s.len() {
        out.push(&s[last..]);
    }

    out
}






fn encode_proofnode_to_termfp(
    n: &ProofNode,
    facts: &HashMap<String, Fp>
) -> Vec<TermFp> {
    match n {
        ProofNode::GoalNode(child) => {
            encode_str_to_termfp(&child.goal, facts)
        }
        _ => vec![TermFp {
            name: Fp::zero(),
            args: vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY],
            fact_hashes:Fp::zero(),
        }],
    }
}

// Building the factmap to get the salts easier
pub fn build_fact_map(facts: &[FactEntry]) -> HashMap<String, Fp> {
    let mut map = HashMap::new();

    for conf in facts {
        // Build the key string
        let key = if conf.args.is_empty() {
            conf.predicate.clone()
        } else {
            format!("{}({})", conf.predicate, conf.args.join(","))
        };
        // Convert salt to Fp
        let salt = to_fp_value(&conf.salt);

        map.insert(key, salt);
    }
    map
}

// CPU RLC counter
pub fn rlc_encode_cpu(tokens: &[Fp], alpha: Fp) -> Fp {
    let mut acc = Fp::zero();
    for &t in tokens {
        acc = acc * alpha + t;
    }
    acc
}

/*pub fn parse_complex_goal(input: &str, facts: &HashMap<String, Fp>) -> Vec<TermFp> {
    // Műveleti jelek prioritási sorrendben
    let operators = [" is ", "=", ">", "<", "*", " div ", "+", "-"];

    let mut results = Vec::new();
    let expr = input.to_string();

    for op in operators {
        if expr.contains(op) {
            let parts: Vec<&str> = expr.split(op).map(|s| s.trim()).collect();

            if parts.len() == 2 {
                // 1️⃣ első művelet → bináris predikátum
                let pred_name = op.trim();
                let term_str = format!("{}({},{})",
                    pred_name, parts[0], parts[1]
                );
                results.push(encode_str_to_termfp(&term_str, facts));

                // 2️⃣ második operandus lehet további műveletes
                let right = parts[1];
                if operators.iter().any(|o| right.contains(o)) {
                    results.extend(parse_complex_goal(right, facts));
                } else {
                    // Egyszerű literál → egyeuritású unary predikátum
                    let term_str = format!("{}({})", pred_name, right);
                    results.push(encode_str_to_termfp(&term_str, facts));
                }
                return results;
            }
        }
    }

    // Nincs benne művelet → egy predikátum
    results.push(encode_str_to_termfp(input, facts));
    results
}*/



fn encode_str_to_termfp_og(input: &str, facts: &HashMap<String, Fp>) -> TermFp {
    let open = input.find('(').unwrap_or(input.len());
    let close = input.rfind(')').unwrap_or(input.len());

    let name_str = input[..open].trim();
    let args_str = if open < close {
        &input[open + 1..close]
    } else {
        ""
    };

    let mut flat_args: Vec<&str> = args_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if name_str == "consumptionClass" && flat_args[1] == "low" {
        flat_args[0] = "_";
    }
    let reconstructed = format!("{}({})", name_str, flat_args.join(","));

    // ✅ Convert to correct 2D args matrix
    let mut args_matrix = vec![vec![Fp::zero(); MAX_PRED_LIST]; MAX_ARITY];
    for (i, val) in flat_args.clone().into_iter().enumerate() {
        if !val.is_empty() {
            args_matrix[i][0] = to_fp_value(val);
        }
    }
    
    let salt = facts.get(&reconstructed).copied().unwrap_or(Fp::zero());
    TermFp {
        name: to_fp_value(name_str),
        args: args_matrix,
        fact_hashes: salt,
    }
}
