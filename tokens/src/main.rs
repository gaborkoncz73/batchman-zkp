use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use common::data;

fn main() -> Result<()> {
    let text = fs::read_to_string("prolog/rules.pl")?;

    let re_rule = Regex::new(r"(?m)^(\w+)\(([^)]*)\)\s*:-\s*(.*)\.")?;
    let re_fact = Regex::new(r"(?m)^(\w+)\(([^)]*)\)\s*\.")?;
    let re_body = Regex::new(r"(\w+)\(([^)]*)\)")?;

    let mut predicates: Vec<data::PredicateTemplate> = Vec::new();
    let mut facts: Vec<data::FactTemplate> = Vec::new();

    let mut seen_heads: HashSet<(String, usize)> = HashSet::new();
    let mut seen_body: HashSet<(String, usize)> = HashSet::new();

    // processing rules
    for cap in re_rule.captures_iter(&text) {
    let head_name = &cap[1];
    let head_args: Vec<_> = cap[2].split(',').map(|s| s.trim()).collect();
    let body = &cap[3];

    let mut children = Vec::new();
    let mut equalities = Vec::new();

    for (i, bcap) in re_body.captures_iter(body).enumerate() {
        let child_name = bcap[1].to_string();
        let child_args: Vec<_> = bcap[2].split(',').map(|s| s.trim()).collect();

        children.push(data::ChildSig {
            name: child_name.clone(),
            arity: child_args.len(),
        });
        seen_body.insert((child_name.clone(), child_args.len()));

        // automatic head–child unifications
        for (j, carg) in child_args.iter().enumerate() {
            if let Some(hpos) = head_args.iter().position(|x| x == carg) {
                equalities.push(data::Equality {
                    left:  data::TermRef { node: 0, arg: hpos },
                    right: data::TermRef { node: i + 1, arg: j },
                });
            }
        }

        // auto child–child equalities
        for (i1, a1) in child_args.iter().enumerate() {
            for (k, other_bcap) in re_body.captures_iter(body).enumerate().skip(i + 1) {
                let other_args: Vec<_> = other_bcap[2].split(',').map(|s| s.trim()).collect();
                for (j, a2) in other_args.iter().enumerate() {
                    if a1 == a2 {
                        equalities.push(data::Equality {
                            left:  data::TermRef { node: i + 1, arg: i1 },
                            right: data::TermRef { node: k + 1, arg: j },
                        });
                    }
                }
            }
        }
    }

    let clause = data::ClauseTemplate { children, equalities };
    let key = (head_name.to_string(), head_args.len());
    seen_heads.insert(key.clone());

    // ✅ now match both name and arity
    if let Some(pred) = predicates
        .iter_mut()
        .find(|p| p.name == head_name && p.arity == head_args.len())
    {
        pred.clauses.push(clause);
    } else {
        predicates.push(data::PredicateTemplate {
            name: head_name.to_string(),
            arity: head_args.len(),
            clauses: vec![clause],
        });
    }
}


    // processing facts
    for cap in re_fact.captures_iter(&text) {
        let name = cap[1].to_string();
        let args: Vec<_> = cap[2].split(',').map(|s| s.trim()).collect();
        let key = (name.clone(), args.len());

        // if it's not head, then it is real fact
        if !seen_heads.contains(&key) {
            facts.push(data::FactTemplate {
                name,
                arity: args.len(),
            });
        }
    }

    // Predicates that appear in the body but not in any head are also treated as facts
    for (name, arity) in seen_body {
        if !seen_heads.contains(&(name.clone(), arity)) {
            if !facts.iter().any(|f| f.name == name && f.arity == arity) {
                facts.push(data::FactTemplate { name, arity });
            }
        }
    }

    let rules = data::RuleTemplateFile { predicates, facts };
    let json = serde_json::to_string_pretty(&rules)?;
    fs::write("input/rules_template.json", json)?;

    println!("rules_template.json is created with the facts and predicates fields");
    Ok(())
}
