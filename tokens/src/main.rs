use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::input_stream::InputStream;
use antlr4rust::tree::{ParseTree, ParseTreeVisitorCompat};

mod parser;
use parser::prologlexer::prologLexer;
use parser::prologparser::{prologParser, ClauseContext};
use parser::prologvisitor::prologVisitor;

// ===== JSON structures =====

#[derive(Debug, Serialize, Clone)]
struct NodeRef {
    node: usize,
    arg: usize,
}

#[derive(Debug, Serialize, Clone)]
struct Equality {
    left: NodeRef,
    right: NodeRef,
}

#[derive(Debug, Serialize, Clone)]
struct Child {
    name: String,
    arity: usize,
}

#[derive(Debug, Serialize, Clone)]
struct Clause {
    children: Vec<Child>,
    equalities: Vec<Equality>,
}

#[derive(Debug, Serialize, Clone)]
struct Predicate {
    name: String,
    arity: usize,
    clauses: Vec<Clause>,
}

#[derive(Debug, Serialize, Clone)]
struct Root {
    predicates: Vec<Predicate>,
    facts: Vec<Child>,
}

// ===== JSON Builder =====

pub struct JsonBuilder {
    pub root: Root,
    temp: (),
}

impl JsonBuilder {
    fn new() -> Self {
        Self {
            root: Root {
                predicates: Vec::new(),
                facts: Vec::new(),
            },
            temp: (),
        }
    }

    /// Split body predicates safely (ignores commas inside parentheses)
    fn split_body_terms(body: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut level: i32 = 0; // <-- explicit type
        let mut current = String::new();
        for c in body.chars() {
            match c {
                '(' => {
                    level += 1;
                    current.push(c);
                }
                ')' => {
                    if level > 0 {
                        level -= 1;
                    }
                    current.push(c);
                }
                ',' if level == 0 => {
                    result.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            result.push(current.trim().to_string());
        }
        result
    }

    /// Parse "f(X,Y)" -> ("f", ["X","Y"])
    fn parse_term(term: &str) -> (String, Vec<String>) {
        let t = term.trim().trim_end_matches('.');

        if let Some(open) = t.find('(') {
            let close = t.rfind(')').unwrap_or(t.len());
            let name = t[..open].trim().to_string();
            let args_raw = &t[open + 1..close];
            let args: Vec<String> = args_raw
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            (name, args)
        } else {
            (t.to_string(), vec![])
        }
    }

    fn process_clause_text(&mut self, text: &str) {
        let t = text.trim();

        if let Some(colon_dash) = t.find(":-") {
            // ===== Rule =====
            let (head_raw, body_raw_with_dot) = t.split_at(colon_dash);
            let head_text = head_raw.trim();
            let body_text = body_raw_with_dot[2..].trim().trim_end_matches('.');

            // Parse head and body
            let (head_name, head_args) = Self::parse_term(head_text);

            // Use the safe body splitter
            let body_parts = Self::split_body_terms(body_text);
            let body_terms: Vec<(String, Vec<String>)> =
                body_parts.iter().map(|b| Self::parse_term(b)).collect();

            // Build children
            let children: Vec<Child> = body_terms
                .iter()
                .map(|(name, args)| Child {
                    name: name.clone(),
                    arity: args.len(),
                })
                .collect();

            // --- Build variable map ---
            let mut varmap: HashMap<String, Vec<NodeRef>> = HashMap::new();
            let mut all_nodes: Vec<Vec<String>> = Vec::new();
            all_nodes.push(head_args.clone());
            for (_, args) in &body_terms {
                all_nodes.push(args.clone());
            }

            for (node_idx, args) in all_nodes.iter().enumerate() {
                for (arg_idx, var) in args.iter().enumerate() {
                    if var.chars().next().map(|c| c.is_uppercase() || c == '_').unwrap_or(false) {
                        varmap
                            .entry(var.clone())
                            .or_default()
                            .push(NodeRef { node: node_idx, arg: arg_idx });
                    }
                }
            }

            // --- Generate equality pairs ---
            let mut equalities = Vec::new();
            for refs in varmap.values() {
                if refs.len() > 1 {
                    let first = &refs[0];
                    for other in refs.iter().skip(1) {
                        equalities.push(Equality {
                            left: first.clone(),
                            right: other.clone(),
                        });
                    }
                }
            }

            // --- Insert into predicate list ---
            let clause = Clause { children, equalities };
            if let Some(pred) = self
                .root
                .predicates
                .iter_mut()
                .find(|p| p.name == head_name && p.arity == head_args.len())
            {
                pred.clauses.push(clause);
            } else {
                self.root.predicates.push(Predicate {
                    name: head_name,
                    arity: head_args.len(),
                    clauses: vec![clause],
                });
            }
        } else if t.ends_with('.') {
            // ===== Fact =====
            let fact_text = t.trim_end_matches('.').trim();
            let (name, args) = Self::parse_term(fact_text);
            let arity = args.len();
            if !self.root.facts.iter().any(|f| f.name == name && f.arity == arity) {
                self.root.facts.push(Child { name, arity });
            }
        }
    }
}

// ===== Implement Visitors =====

impl<'input> prologVisitor<'input> for JsonBuilder {
    fn visit_clause(&mut self, ctx: &ClauseContext<'input>) {
        let txt = ctx.get_text();
        self.process_clause_text(&txt);
    }
}

impl<'input> ParseTreeVisitorCompat<'input> for JsonBuilder {
    type Node = parser::prologparser::prologParserContextType;
    type Return = ();

    fn temp_result(&mut self) -> &mut Self::Return {
        &mut self.temp
    }
}

// ===== main =====

fn main() -> Result<()> {
    let input_text =
        std::fs::read_to_string("prolog/rules.pl").expect("Failed to read prolog/rules.pl");

    let input = InputStream::new(input_text.as_str());
    let lexer = prologLexer::new(input);
    let token_source = CommonTokenStream::new(lexer);
    let mut parser = prologParser::new(token_source);

    let tree = parser.p_text().unwrap();

    let mut builder = JsonBuilder::new();
    builder.visit(&*tree);

    let json = serde_json::to_string_pretty(&builder.root)?;
    println!("{json}");
    Ok(())
}
