use anyhow::Result;
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;

#[derive(Debug, Serialize)]
struct RuleTemplateFile {
    predicates: Vec<PredicateTemplate>,
    facts: Vec<FactTemplate>,
}

#[derive(Debug, Serialize)]
struct PredicateTemplate {
    name: String,
    arity: usize,
    clauses: Vec<ClauseTemplate>,
}

#[derive(Debug, Serialize)]
struct ClauseTemplate {
    children: Vec<ChildSig>,
    equalities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ChildSig {
    name: String,
    arity: usize,
}

#[derive(Debug, Serialize)]
struct FactTemplate {
    name: String,
    arity: usize,
}

fn main() -> Result<()> {
    let text = fs::read_to_string("prolog/rules.pl")?;

    let re_rule = Regex::new(r"(?m)^(\w+)\(([^)]*)\)\s*:-\s*(.*)\.")?;
    let re_fact = Regex::new(r"(?m)^(\w+)\(([^)]*)\)\s*\.")?;
    let re_body = Regex::new(r"(\w+)\(([^)]*)\)")?;

    let mut predicates: Vec<PredicateTemplate> = Vec::new();
    let mut facts: Vec<FactTemplate> = Vec::new();

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

            children.push(ChildSig {
                name: child_name.clone(),
                arity: child_args.len(),
            });
            seen_body.insert((child_name.clone(), child_args.len()));

            // automatic unification generation
            for (j, carg) in child_args.iter().enumerate() {
                if let Some(hpos) = head_args.iter().position(|x| x == carg) {
                    equalities.push(format!("HeadArg({})=ChildArg({}, {})", hpos, i, j));
                }
            }
        }

        let clause = ClauseTemplate { children, equalities };
        let key = (head_name.to_string(), head_args.len());
        seen_heads.insert(key.clone());

        if let Some(pred) = predicates.iter_mut().find(|p| p.name == head_name) {
            pred.clauses.push(clause);
        } else {
            predicates.push(PredicateTemplate {
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
            facts.push(FactTemplate {
                name,
                arity: args.len(),
            });
        }
    }

    // Predicates that appear in the body but not in any head are also treated as facts
    for (name, arity) in seen_body {
        if !seen_heads.contains(&(name.clone(), arity)) {
            if !facts.iter().any(|f| f.name == name && f.arity == arity) {
                facts.push(FactTemplate { name, arity });
            }
        }
    }

    let rules = RuleTemplateFile { predicates, facts };
    let json = serde_json::to_string_pretty(&rules)?;
    fs::write("input/rules_template.json", json)?;

    println!("rules_template.json is created with the facts and predicates fields");
    Ok(())
}
