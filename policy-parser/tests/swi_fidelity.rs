//! Layout-sensitive syntax must read exactly as in SWI-Prolog.
//!
//! `CASES` holds what SWI-Prolog 9.2.9 prints for each input with
//! `read_term/3` + `write_term(T, [quoted(true), ignore_ops(true)])`, with
//! variables shown as `_`; `None` means SWI-Prolog raises a syntax error.
//! `cases_match_live_swipl` re-checks the table whenever `swipl` is on PATH.

use std::io::Write;
use std::process::{Command, Stdio};

use common::policy_ir::Term;
use policy_parser::{parse_goal, parse_policy};

const CASES: &[(&str, Option<&str>)] = &[
    // Negative number literals: '-' directly followed by a number.
    ("-1", Some("-1")),
    ("- 1", Some("-(1)")),
    ("-1.5", Some("-1.5")),
    ("-0x10", Some("-16")),
    ("-0b101", Some("-5")),
    ("-0'a", Some("-97")),
    ("-0", Some("0")),
    ("+1", Some("+(1)")),
    ("-1^2", Some("^(-1,2)")),
    ("- 1^2", Some("-(^(1,2))")),
    ("-1**2", Some("**(-1,2)")),
    ("2 ** -1", Some("**(2,-1)")),
    ("2 ^ -1", Some("^(2,-1)")),
    ("2 ** - 1", None),
    ("2 ** -(1)", Some("**(2,-(1))")),
    ("-1*2", Some("*(-1,2)")),
    ("- - 1", Some("-(-(1))")),
    ("- -1", Some("-(-1)")),
    ("-(-1)", Some("-(-1)")),
    ("f(-1)", Some("f(-1)")),
    ("f(- 1)", Some("f(-(1))")),
    ("X is -1 + 2", Some("is(_,+(-1,2))")),
    // Infix minus is unaffected by the negative-literal rule.
    ("a-1", Some("-(a,1)")),
    ("a - 1", Some("-(a,1)")),
    ("a -1", Some("-(a,1)")),
    ("[a -1]", Some("[-(a,1)]")),
    ("a- -1", Some("-(a,-1)")),
    ("1 - (2)", Some("-(1,2)")),
    ("1 -(2)", Some("-(1,2)")),
    // Prefix operator vs. functional notation of an operator name.
    ("-a", Some("-(a)")),
    ("- a", Some("-(a)")),
    ("-(a)", Some("-(a)")),
    ("-(1)", Some("-(1)")),
    ("-(1,2)", Some("-(1,2)")),
    ("- (1,2)", Some("-(','(1,2))")),
    ("-(1)^2", Some("^(-(1),2)")),
    ("- (1)^2", Some("-(^(1,2))")),
    ("- (a,b)^2", Some("-(^(','(a,b),2))")),
    ("-(-(1,2))", Some("-(-(1,2))")),
    ("f(a,-(1,2))", Some("f(a,-(1,2))")),
    ("a = -(1)", Some("=(a,-(1))")),
    ("=(a,b)", Some("=(a,b)")),
    ("X=(a,b)", Some("=(_,','(a,b))")),
    ("\\+(a) = b", Some("=(\\+(a),b)")),
    ("\\+ (a) = b", Some("\\+(=(a,b))")),
    ("\\+(a)", Some("\\+(a)")),
    ("\\(a)", Some("\\(a)")),
    ("\\ (a)", Some("\\(a)")),
    ("mod(7,2)", Some("mod(7,2)")),
    ("7 mod(2)", Some("mod(7,2)")),
    ("is(X,1)", Some("is(_,1)")),
    ("<(a,b)", Some("<(a,b)")),
    (":(a,b)", Some(":(a,b)")),
    (";(a,b)", Some(";(a,b)")),
    ("->(a,b)", Some("->(a,b)")),
    // A functor must be followed immediately by '('.
    ("foo (a)", None),
    ("foo(a) (b)", None),
    ("foo/**/(a)", None),
    ("foo/*c*/(a)", None),
    // '/*' opens a comment at the start of a token, but not inside a
    // symbol-char sequence.
    ("a/**/= b", Some("=(a,b)")),
    ("a/*x*/=b", Some("=(a,b)")),
    ("a = - /**/ 1", Some("=(a,-(1))")),
    ("f(a)/**/", Some("f(a)")),
    ("a =/**/ b", None),
    ("a=/*x*/b", None),
    ("a/**/-/**/1", None),
    ("-/**/1", None),
];

/// Formats a term like SWI-Prolog's `write_term/2` with `quoted(true)` and
/// `ignore_ops(true)`, covering the terms that occur in `CASES`.
fn swi_canonical(term: &Term) -> String {
    match term {
        Term::Variable { .. } => "_".to_owned(),
        Term::Integer { decimal } => decimal.clone(),
        Term::Float { lexeme } => lexeme.clone(),
        Term::Atom { value } => quoted_atom(value),
        Term::Compound { functor, args } => format!(
            "{}({})",
            quoted_atom(functor),
            args.iter().map(swi_canonical).collect::<Vec<_>>().join(",")
        ),
        Term::List { items, tail } => {
            let items = items.iter().map(swi_canonical).collect::<Vec<_>>();
            match tail {
                Some(tail) => format!("[{}|{}]", items.join(","), swi_canonical(tail)),
                None => format!("[{}]", items.join(",")),
            }
        }
        other => panic!("swi_canonical does not cover {other:?}"),
    }
}

fn quoted_atom(atom: &str) -> String {
    let alphanumeric = atom.starts_with(|c: char| c.is_ascii_lowercase())
        && atom.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    let symbolic = !atom.is_empty() && atom.chars().all(|c| "#$&*+-./:<=>?@^~\\".contains(c));
    if alphanumeric || symbolic || matches!(atom, "!" | ";" | "[]" | "{}") {
        atom.to_owned()
    } else {
        format!("'{}'", atom.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

#[test]
fn layout_sensitive_terms_read_as_in_swi_prolog() {
    let mut failures = Vec::new();
    for (source, expected) in CASES {
        let actual = parse_goal(source, "case").map(|term| swi_canonical(&term));
        let matches = match (expected, &actual) {
            (Some(expected), Ok(actual)) => expected == actual,
            (None, Err(_)) => true,
            _ => false,
        };
        if !matches {
            failures.push(format!("{source:?}: SWI {expected:?}, parser {actual:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn layout_errors_report_their_position() {
    let error = parse_goal("foo/**/(a)", "goal").unwrap_err().to_string();
    assert!(error.contains("1:7") && error.contains("followed immediately by '('"), "{error}");
    let error = parse_goal("2 ** - 1", "goal").unwrap_err().to_string();
    assert!(error.contains("1:5") && error.contains("priority clash"), "{error}");
}

#[test]
fn clause_layout_matches_swi_prolog() {
    // SWI-Prolog rejects all of these when consulting the file.
    for source in [
        "p(1).q(1).\n",
        "foo (a) :- true.\n",
        "foo/**/(a).\n",
        "p :- X = a.b.\n",
    ] {
        assert!(parse_policy(source, "case.pl").is_err(), "{source:?}");
    }
    // ...and accepts these.
    let policy = parse_policy("p(-1).%c\np(- 1).\np :- q.\n", "case.pl").unwrap();
    let p1 = policy.predicates.iter().find(|p| p.indicator.arity == 1).unwrap();
    let clauses = &p1.clauses;
    assert_eq!(swi_canonical(&clauses[0].head.args[0]), "-1");
    assert_eq!(swi_canonical(&clauses[1].head.args[0]), "-(1)");
}

/// Confirms `CASES` against the installed SWI-Prolog; skipped without `swipl`.
#[test]
fn cases_match_live_swipl() {
    if Command::new("swipl").arg("--version").output().is_err() {
        eprintln!("swipl not found; skipping live SWI-Prolog comparison");
        return;
    }
    let goal = "catch((read_term(user_input, T, []), \
                write_term(T, [quoted(true), ignore_ops(true)])), _, write('<error>')), \
                nl, halt.";
    let mut failures = Vec::new();
    for (source, expected) in CASES {
        let mut child = Command::new("swipl")
            .args(["-q", "-g", goal])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        writeln!(child.stdin.take().unwrap(), "{source} .").unwrap();
        let output = String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap();
        let swi = anonymize_variables(output.trim());
        let expected = expected.unwrap_or("<error>");
        if swi != expected {
            failures.push(format!("{source:?}: table {expected:?}, swipl {swi:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Replaces SWI-Prolog's `_123` / `_G123` variable names with `_`.
fn anonymize_variables(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        output.push(c);
        if c == '_' {
            while chars.peek().is_some_and(|c| c.is_ascii_alphanumeric()) {
                chars.next();
            }
        }
    }
    output
}
