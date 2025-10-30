use antlr4rust::{common_token_stream::CommonTokenStream, InputStream};
use antlr4rust::tree::{ParseTree, ParseTreeVisitorCompat};
use std::collections::{HashMap, HashSet};

pub mod parser;
use parser::prologlexer::prologLexer;
use parser::prologparser::*;
use parser::prologvisitor::prologVisitor;

use serde::Serialize;

// ------------------ Kimeneti JSON model ------------------

#[derive(Serialize)]
struct OutRoot {
    predicates: Vec<OutPredicate>,
}

#[derive(Serialize, Clone)]
struct OutPredicate {
    name: String,
    arity: usize,
    clauses: Vec<OutClause>,
}

#[derive(Serialize, Clone)]
struct OutClause {
    children: Vec<ChildPred>,
    equalities: Vec<Equality>,
    builtins: Vec<BuiltInCall>,
}

#[derive(Serialize, Clone)]
struct ChildPred {
    name: String,
    arity: usize,
}

#[derive(Serialize, Clone)]
struct NodeArgRef {
    node: usize, // 0 = head, 1.. = body pred index + 1
    arg: usize,
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
enum RightSide {
    Ref(NodeArgRef),
    Atom(String),
}

#[derive(Serialize, Clone)]
struct Equality {
    left: NodeArgRef,
    right: RightSide,
}

// ----- Built-in operátorfa -----
#[derive(Serialize, Clone)]
struct BuiltInCall {
    operator: String,
    operands: Vec<Operand>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind")] // egyértelmű JSON reprezentáció
enum Operand {
    Ref { node: usize, arg: usize }, // hivatkozás valamely predikátum egy argumentumára
    Atom { value: String },          // szám/atom literál
    Var { name: String },            // csak testbeli lokális változó, nincs kötése arg pozícióhoz
    BuiltIn { operator: String, operands: Vec<Operand> }, // beágyazott művelet
}

// ------------------ Belső (egyszerű) AST ------------------

#[derive(Debug, Clone)]
enum Term {
    Atom(String),
    Var(String),
    Predicate { name: String, args: Vec<Term> },
}

impl Term {
    fn arity(&self) -> usize {
        match self {
            Term::Predicate { args, .. } => args.len(),
            _ => 0,
        }
    }
}

// ------------------ Clause gyűjtő ------------------

#[derive(Clone)]
struct Clause {
    head: Term,       // kötelezően Predicate
    body: Vec<Term>,  // Term-ek; ebből szűrjük a gyerek predikátumokat és built-ineket
}

// ------------------ Visitor → AST builder ------------------

struct AstBuilder {
    clauses: Vec<Clause>,
    _unit: (),
}

impl AstBuilder {
    fn new() -> Self {
        Self { clauses: vec![], _unit: () }
    }
    fn parse_list(&self, s: &str) -> Term {
    // Remove [  ] 
    let inner = &s[1..s.len() - 1];

    if inner.is_empty() {
        return Term::Atom("[]".into());
    }

    if let Some(pos) = inner.find('|') {
        let head = inner[..pos].trim();
        let tail = inner[pos+1..].trim();
        return Term::Predicate {
            name: ".".into(),
            args: vec![
                self.parse_term_str(head),
                self.parse_term_str(tail),
            ],
        };
    }

    // comma separated list: [1,2,3]
    let parts = inner.split(',').map(|p| p.trim()).collect::<Vec<_>>();

    let mut t = Term::Atom("[]".into());
    for elem in parts.into_iter().rev() {
        t = Term::Predicate {
            name: ".".into(),
            args: vec![self.parse_term_str(elem), t],
        };
    }
    t
}

fn parse_term_str(&self, s: &str) -> Term {
    // Shortcut: if list, handle here
    if s.starts_with('[') && s.ends_with(']') {
        return self.parse_list(s);
    }
    // fallback to ANTLR node parse helper
    Term::Atom(s.into())
}
    fn parse_term(&self, ctx: &TermContextAll) -> Term {
        use TermContextAll::*;
        // ha list literal
        let text = ctx.get_text();
        if text.starts_with('[') && text.ends_with(']') {
            return self.parse_list(&text);
        }

        match ctx {
            VariableContext(v) => Term::Var(v.get_text()),
            Integer_termContext(i) => Term::Atom(i.get_text()),
            FloatContext(f) => Term::Atom(f.get_text()),
            Atom_termContext(a) => Term::Atom(a.get_text()),
            List_termContext(l) => {
                // Listát most stringként viszünk tovább
                Term::Atom(l.get_text())
            }
            Compound_termContext(c) => {
                // f(a, X, ...)
                let name = c.atom().unwrap().get_text();
                let args = c
                    .termlist().unwrap()
                    .term_all()
                    .into_iter()
                    .map(|t| self.parse_term(&t))
                    .collect();
                Term::Predicate { name, args }
            }
            Binary_operatorContext(b) => {
                // term operator_ term  ==>  operator(term, term)
                let left = self.parse_term(&b.term(0).unwrap());
                let right = self.parse_term(&b.term(1).unwrap());
                let op = b.operator_().unwrap().get_text();
                Term::Predicate { name: op, args: vec![left, right] }
            }
            Unary_operatorContext(u) => {
                // operator_ term  ==> operator(term)
                let arg = self.parse_term(&u.term().unwrap());
                let op = u.operator_().unwrap().get_text();
                Term::Predicate { name: op, args: vec![arg] }
            }
            _ => Term::Atom(ctx.get_text()),
        }
    }
}

impl<'input> ParseTreeVisitorCompat<'input> for AstBuilder {
    type Node = prologParserContextType;
    type Return = ();
    fn temp_result(&mut self) -> &mut Self::Return { &mut self._unit }
}

impl<'input> prologVisitor<'input> for AstBuilder {
    fn visit_fact(&mut self, ctx: &FactContext<'input>) {
        let head = self.parse_term(&ctx.term().unwrap());
        self.clauses.push(Clause { head, body: vec![] });
        self.visit_children(ctx);
    }

    fn visit_rule_(&mut self, ctx: &Rule_Context<'input>) {
        let head = self.parse_term(&ctx.head().unwrap().term().unwrap());

        if let Some(body) = ctx.body() {
            // A grammar a diszjunkciót több termlist-tel adja vissza; mind külön klóz
            for tl in body.termlist_all() {
                let terms = tl.term_all()
                    .into_iter()
                    .map(|t| self.parse_term(&t))
                    .collect::<Vec<_>>();
                self.clauses.push(Clause { head: head.clone(), body: terms });
            }
        }
        self.visit_children(ctx);
    }
}

// ------------------ Clause → kért JSON szerkezet ------------------

fn to_output(clauses: Vec<Clause>) -> OutRoot {
    use std::collections::HashMap;

    let mut map: HashMap<(String, usize), OutPredicate> = HashMap::new();

    // Beépített operátorok
    let builtin_ops: HashSet<&'static str> = [
        // arithmetic & evaluation
        "is", "+", "-", "*", "/", "div", "mod", "//", "rem", "**", "^", "<<", ">>",
        // comparisons / relations
        "<", ">", ">=", "=:=", "=\\=",
        // unification & (in)equality
        "=", "\\=", "==", "\\==", "=..",
        // standard order
        "@<", "@>", "@>=",
        // control / logical connectives (optional)
        "->", ",", ";",
        // negation
        "\\+",
    ].into_iter().collect();

    for cl in clauses {
        // Head
        let (hname, hargs) = match &cl.head {
            Term::Predicate { name, args } => (name.clone(), args.clone()),
            Term::Atom(a) => (a.clone(), vec![]),
            Term::Var(v) => (v.clone(), vec![]),
        };
        let harity = hargs.len();

        // Body felosztása children vs builtins
        let mut children: Vec<(String, usize, Vec<Term>)> = vec![];
        let mut builtin_terms: Vec<Term> = vec![];

        for t in &cl.body {
            if let Term::Predicate { name, args } = t.clone() {
                if builtin_ops.contains(name.as_str()) {
                    builtin_terms.push(Term::Predicate { name, args });
                } else {
                    children.push((name, args.len(), args));
                }
            }
        }

        // --- Equalities építése ---
        #[derive(Clone)]
        struct Occ { node: usize, arg: usize }

        let mut var_pos: HashMap<String, Vec<Occ>> = HashMap::new();
        let mut atom_pos: Vec<(Occ, String)> = vec![];

        let mut push_term_occ = |node_idx: usize, args: &Vec<Term>| {
            for (i, a) in args.iter().enumerate() {
                match a {
                    Term::Var(v) => {
                        var_pos.entry(v.clone()).or_default().push(Occ { node: node_idx, arg: i });
                    }
                    Term::Atom(s) => {
                        atom_pos.push((Occ { node: node_idx, arg: i }, s.clone()));
                    }
                    Term::Predicate { .. } => { /* összetett argumentum kihagyva equalities-hez */ }
                }
            }
        };

        push_term_occ(0, &hargs);
        for (child_idx, (_nm, _ar, args)) in children.iter().enumerate() {
            push_term_occ(1 + child_idx, args);
        }

        // Literal equality a testbeli '=' alapján (var = atom | atom = var)
        for op in &builtin_terms {
            if let Term::Predicate { name, args } = op {
                if name == "=" && args.len() == 2 {
                    match (&args[0], &args[1]) {
                        (Term::Var(v), Term::Atom(lit)) | (Term::Atom(lit), Term::Var(v)) => {
                            if let Some(occs) = var_pos.get(v) {
                                if let Some(first) = occs.first() {
                                    atom_pos.push((Occ { node: first.node, arg: first.arg }, lit.clone()));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut equalities: Vec<Equality> = vec![];
        for (_v, occs) in &var_pos {
            if occs.len() > 1 {
                let base = &occs[0];
                for other in occs.iter().skip(1) {
                    equalities.push(Equality {
                        left: NodeArgRef { node: base.node, arg: base.arg },
                        right: RightSide::Ref(NodeArgRef { node: other.node, arg: other.arg }),
                    });
                }
            }
        }
        for (occ, lit) in atom_pos {
            equalities.push(Equality {
                left: NodeArgRef { node: occ.node, arg: occ.arg },
                right: RightSide::Atom(lit),
            });
        }

        // Children minimal
        let children_min = children.iter()
            .map(|(n, a, _)| ChildPred { name: n.clone(), arity: *a })
            .collect::<Vec<_>>();

        // --- Builtin operandusfa ---
        let resolve_var_ref = |vname: &str| -> Option<NodeArgRef> {
            var_pos.get(vname).and_then(|occs| occs.first().map(|o| NodeArgRef { node: o.node, arg: o.arg }))
        };

        fn term_to_operand(
            t: &Term,
            is_builtin: &dyn Fn(&str) -> bool,
            resolve: &dyn Fn(&str) -> Option<NodeArgRef>,
        ) -> Operand {
            match t {
                Term::Atom(s) => Operand::Atom { value: s.clone() },
                Term::Var(v) => {
                    if let Some(r) = resolve(v) { Operand::Ref { node: r.node, arg: r.arg } }
                    else { Operand::Var { name: v.clone() } }
                }
                Term::Predicate { name, args } => {
                    if is_builtin(name) {
                        let ops = args.iter().map(|a| term_to_operand(a, is_builtin, resolve)).collect();
                        Operand::BuiltIn { operator: name.clone(), operands: ops }
                    } else {
                        Operand::Atom { value: format!("{}(…)", name) }
                    }
                }
            }
        }

        let mut builtins: Vec<BuiltInCall> = vec![];
        for bt in builtin_terms {
            if let Term::Predicate { name, args } = bt {
                let operands = args.iter().map(|a| term_to_operand(a, &|n| builtin_ops.contains(n), &resolve_var_ref)).collect();
                builtins.push(BuiltInCall { operator: name, operands });
            }
        }

        let out_clause = OutClause { children: children_min, equalities, builtins };

        map.entry((hname.clone(), harity))
            .and_modify(|op| op.clauses.push(out_clause.clone()))
            .or_insert_with(|| OutPredicate { name: hname, arity: harity, clauses: vec![out_clause] });
    }

    let mut predicates = map.into_values().collect::<Vec<_>>();
    predicates.sort_by(|a,b| a.name.cmp(&b.name).then(a.arity.cmp(&b.arity)));
    OutRoot { predicates }
}

// ------------------ main ------------------

fn main() {
    // Prolog forrás
    let input = std::fs::read_to_string("./prolog/policy.pl").expect("read failed");

    // ANTLR: lex + parse
    let input_stream = InputStream::new(input.as_str());
    let lexer = prologLexer::new(input_stream);
    let token_stream = CommonTokenStream::new(lexer);
    let mut parser = prologParser::new(token_stream);

    let tree = parser.p_text().expect("parse");

    // AST építés
    let mut builder = AstBuilder::new();
    builder.visit_p_text(&tree);

    // JSON-összerakás
    let out = to_output(builder.clauses);
    let json = serde_json::to_string_pretty(&out).expect("json serialize");

    // ✅ JSON kiírás fájlba
    let file_path = "parsed.json";
    std::fs::write(file_path, &json).expect("write failed");

    // ✅ és konzolra is csak jelzés
    println!("✅ JSON saved to {file_path}");
}
