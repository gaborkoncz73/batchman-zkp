//! Prolog text → [`Term`], independent of the policy dialect and the circuit.

mod literals;
mod terms;

use std::cell::RefCell;
use std::rc::Rc;

use antlr4rust::error_listener::ErrorListener;
use antlr4rust::errors::ANTLRError;
use antlr4rust::parser::Parser;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token::{TOKEN_EOF, Token};
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::{InputStream, common_token_stream::CommonTokenStream};
use anyhow::{Context, Result, anyhow, bail};
use common::policy_ir::Term;

use crate::parser::prologlexer::prologLexer;
use crate::parser::prologparser::prologParser;

pub(crate) use terms::{build_clause, build_directive};

/// Collects every lexer/parser diagnostic as `line:column: message`.
#[derive(Clone, Default)]
struct CollectingErrorListener {
    diagnostics: Rc<RefCell<Vec<String>>>,
}

impl<'input, T> ErrorListener<'input, T> for CollectingErrorListener
where
    T: Recognizer<'input>,
{
    fn syntax_error(
        &self,
        _recognizer: &T,
        _offending_symbol: Option<&<T::TF as TokenFactory<'input>>::Inner>,
        line: isize,
        column: isize,
        message: &str,
        _error: Option<&ANTLRError>,
    ) {
        self.diagnostics
            .borrow_mut()
            .push(format!("{}:{}: {message}", line.max(0), column.max(0)));
    }
}

pub(crate) type PolicyParser<'a> =
    prologParser<'a, CommonTokenStream<'a, prologLexer<'a, InputStream<&'a str>>>>;

/// Runs one start rule with collected diagnostics; the whole input must be consumed.
pub(crate) fn run_parser<'a, T>(
    source: &'a str,
    source_name: &str,
    start: impl FnOnce(&mut PolicyParser<'a>) -> std::result::Result<T, Box<ANTLRError>>,
) -> Result<T> {
    let errors = CollectingErrorListener::default();
    let mut lexer = prologLexer::new(InputStream::new(source));
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(errors.clone()));
    let mut parser = prologParser::new(CommonTokenStream::new(lexer));
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(errors.clone()));

    let tree_result = start(&mut parser);
    let diagnostics = errors.diagnostics.borrow();
    if !diagnostics.is_empty() {
        bail!(
            diagnostics
                .iter()
                .map(|diagnostic| format!("{source_name}:{diagnostic}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    let tree = tree_result.map_err(|error| anyhow!("{source_name}: parser failure: {error:?}"))?;
    let trailing = parser.get_current_token();
    if trailing.get_token_type() != TOKEN_EOF {
        bail!(
            "{}:{}:{}: unexpected trailing input {:?}",
            source_name,
            trailing.get_line(),
            trailing.get_column(),
            trailing.get_text()
        );
    }
    Ok(tree)
}

/// Parses one proof-tree goal with the same grammar and AST builder as policy
/// bodies. The complete input must be consumed; partial parses are errors.
pub fn parse_goal(source: &str, source_name: &str) -> Result<Term> {
    if source.trim().is_empty() {
        bail!("{source_name}: goal must not be empty");
    }

    let tree = run_parser(source, source_name, |parser| {
        parser.body().map_err(Box::new)
    })?;
    terms::build_goal(&tree)
}

fn required<T>(value: Option<T>, description: &str) -> Result<T> {
    value.with_context(|| format!("ANTLR parse tree is missing {description}"))
}
