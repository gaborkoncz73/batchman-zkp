//! Policy files: the clauses of a `.pl` source.

use std::collections::BTreeMap;

use antlr4rust::parser_rule_context::ParserRuleContext;
use antlr4rust::token::Token;
use anyhow::{Result, anyhow, bail};
use policy_ir::{Clause, PolicyFile, PredicateDef, PredicateIndicator, Term};

use crate::parser::prologparser::*;
use crate::syntax::{build_clause, build_directive, run_parser};

/// Parses a policy source into its predicates.
///
/// The `module/2` directive only controls SWI-Prolog visibility (the
/// meta-interpreter needs every predicate exported), so it is accepted and
/// dropped. Any other directive could change the meaning of the clauses and is
/// rejected.
pub fn parse_policy(source: &str, source_name: &str) -> Result<PolicyFile> {
    let tree = run_parser(source, source_name, |parser| {
        parser.p_text().map_err(Box::new)
    })?;

    let mut clauses = BTreeMap::<PredicateIndicator, Vec<Clause>>::new();
    for item in tree.source_item_all() {
        if let Some(directive) = item.directive() {
            let body = build_directive(&directive)?;
            if !matches!(&body, Term::Compound { functor, args } if functor == "module" && args.len() == 2)
            {
                bail!(
                    "{source_name}:{}:{}: only the module/2 directive is supported",
                    directive.start().get_line(),
                    directive.start().get_column()
                );
            }
        } else if let Some(clause) = item.clause() {
            let (predicate, clause) = build_clause(&clause)?;
            clauses.entry(predicate).or_default().push(clause);
        } else {
            bail!("{source_name}: internal parser error: empty source item");
        }
    }

    let policy = PolicyFile {
        predicates: clauses
            .into_iter()
            .map(|(indicator, clauses)| PredicateDef { indicator, clauses })
            .collect(),
    };
    policy.validate().map_err(|error| anyhow!(error))?;
    Ok(policy)
}

pub fn to_canonical_json(policy: &PolicyFile) -> Result<String> {
    policy.validate().map_err(|error| anyhow!(error))?;
    let mut json = serde_json::to_string_pretty(policy)?;
    json.push('\n');
    Ok(json)
}
