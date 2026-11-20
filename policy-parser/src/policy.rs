//! Policy files: the clauses and the single `module/2` directive of a `.pl` source.

use std::collections::BTreeMap;

use antlr4rust::parser_rule_context::ParserRuleContext;
use antlr4rust::token::Token;
use anyhow::{Context, Result, anyhow, bail};
use common::policy_ir::{
    Clause, ModuleDecl, POLICY_DIALECT, POLICY_SCHEMA_VERSION, PolicyFile, PredicateDef,
    PredicateIndicator, Term,
};

use crate::parser::prologparser::*;
use crate::syntax::{build_clause, build_directive, run_parser};

pub fn parse_policy(source: &str, source_name: &str) -> Result<PolicyFile> {
    let tree = run_parser(source, source_name, |parser| {
        parser.p_text().map_err(Box::new)
    })?;

    let mut module = None;
    let mut directives = Vec::new();
    let mut clauses = BTreeMap::<PredicateIndicator, Vec<Clause>>::new();
    for item in tree.source_item_all() {
        if let Some(directive) = item.directive() {
            let at = format!(
                "{source_name}:{}:{}",
                directive.start().get_line(),
                directive.start().get_column()
            );
            let (body, has_variables) = build_directive(&directive)?;
            if has_variables {
                bail!("{at}: variables in directives are not supported by {POLICY_DIALECT}");
            }
            if module.is_some() {
                bail!("{at}: multiple module/2 directives are not allowed");
            }
            module = Some(module_decl(&body).with_context(|| {
                format!("{at}: unsupported directive in {POLICY_DIALECT}: {body:?}")
            })?);
            directives.push(body);
        } else if let Some(clause) = item.clause() {
            let clause = build_clause(&clause)?;
            let predicate = clauses.entry(clause.head.predicate.clone()).or_default();
            predicate.push(Clause {
                ordinal: predicate.len(),
                ..clause
            });
        } else {
            bail!("{source_name}: internal parser error: empty source item");
        }
    }

    let policy = PolicyFile {
        schema_version: POLICY_SCHEMA_VERSION,
        dialect: POLICY_DIALECT.to_owned(),
        module,
        directives,
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

/// Normalizes `module(Name, [Name/Arity, ...])`, the only supported directive.
fn module_decl(directive: &Term) -> Result<ModuleDecl> {
    let Term::Compound { functor, args } = directive else {
        bail!("expected module/2");
    };
    let [Term::Atom { value: name }, Term::List { items, tail: None }] = args.as_slice() else {
        bail!("expected module(Name, [Name/Arity, ...])");
    };
    if functor != "module" {
        bail!("expected module/2");
    }
    let exports = items
        .iter()
        .map(|export| {
            let Term::Compound { functor, args } = export else {
                bail!("module export must be a Name/Arity predicate indicator");
            };
            let [Term::Atom { value: name }, Term::Integer { decimal }] = args.as_slice() else {
                bail!("module export must be a Name/Arity predicate indicator");
            };
            if functor != "/" {
                bail!("module export must be a Name/Arity predicate indicator");
            }
            let arity = decimal
                .parse()
                .context("predicate-indicator arity does not fit usize")?;
            Ok(PredicateIndicator {
                name: name.clone(),
                arity,
            })
        })
        .collect::<Result<_>>()?;
    Ok(ModuleDecl {
        name: name.clone(),
        exports,
    })
}
