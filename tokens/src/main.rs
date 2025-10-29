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
    body: Vec<Term>,  // Term-ek; ebből szűrjük a gyerek predikátumokat
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

    fn parse_term(&self, ctx: &TermContextAll) -> Term {
        use TermContextAll::*;
        match ctx {
            VariableContext(v) => Term::Var(v.get_text()),
            Integer_termContext(i) => Term::Atom(i.get_text()),
            Atom_termContext(a) => Term::Atom(a.get_text()),
            List_termContext(l) => {
                // A listát most szó szerinti atomként visszük (string literal)
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
            // Bármi más → nyers text atomként
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
    // Csoportosítás (head név, arity) szerint
    let mut map: HashMap<(String, usize), OutPredicate> = HashMap::new();

    for cl in clauses {
        // Head kell hogy Predicate legyen
        let (hname, hargs) = match &cl.head {
            Term::Predicate { name, args } => (name.clone(), args.clone()),
            Term::Atom(a) => (a.clone(), vec![]),
            Term::Var(v) => (v.clone(), vec![]),
        };
        let harity = hargs.len();

        // Body-ból a "children" predikátumok: csak név + arity (nem visszük az args-okat)
        // Operátorokat NE vegyük fel childként (ezekből equality készülhet)
        let op_set: HashSet<&'static str> = [
            "=", "is", "\\=", "==", "\\==", "@<", "@=<", "@>", "@>=",
            "=..", "=:=", "=\\=", "<", "=<", ">", ">=", "->", ",", ";"
        ].into_iter().collect();

        let mut children: Vec<(String, usize, Vec<Term>)> = vec![];
        let mut raw_ops: Vec<Term> = vec![]; // ide tesszük az "=" stb. műveleteket

        for t in &cl.body {
            if let Term::Predicate { name, args } = t.clone() {
                if op_set.contains(name.as_str()) {
                    raw_ops.push(Term::Predicate { name, args });
                } else {
                    children.push((name, args.len(), args));
                }
            }
        }

        // --- Equalities építése ---
        // Node indexelés: 0 = head, 1.. = children sorrendben
        // Gyűjtsük össze minden argumentum helyét: var vagy atom
        #[derive(Clone)]
        struct Occ { node: usize, arg: usize }

        let mut var_pos: HashMap<String, Vec<Occ>> = HashMap::new();
        let mut atom_pos: Vec<(Occ, String)> = vec![];

        // helper a head/child args bejárásához
        let mut push_term_occ = |node_idx: usize, args: &Vec<Term>| {
            for (i, a) in args.iter().enumerate() {
                match a {
                    Term::Var(v) => {
                        var_pos.entry(v.clone()).or_default().push(Occ { node: node_idx, arg: i });
                    }
                    Term::Atom(s) => {
                        atom_pos.push((Occ { node: node_idx, arg: i }, s.clone()));
                    }
                    // összetett argumentumot most literálként nem írjuk ki; ha mégis előfordul,
                    // itt stringgé lehetne alakítani a forrás szövegéből – de a jelen policy.pl-ben nem kell
                    Term::Predicate { .. } => { /* ignoráljuk a literal-equality szempontból */ }
                }
            }
        };

        push_term_occ(0, &hargs);
        for (child_idx, (_nm, _ar, args)) in children.iter().enumerate() {
            push_term_occ(1 + child_idx, args);
        }

        // a testbeli "=" műveletekből is generáljunk literal equality-t
        // csak azokat, ahol var = atom vagy atom = var
        for op in raw_ops {
            if let Term::Predicate { name, args } = op {
                if name == "=" && args.len() == 2 {
                    match (&args[0], &args[1]) {
                        (Term::Var(v), Term::Atom(lit)) | (Term::Atom(lit), Term::Var(v)) => {
                            if let Some(occs) = var_pos.get(v) {
                                // az első előfordulást kötjük a literálhoz
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

        // Végső equalities:
        // 1) Változók: csillag a legelső előfordulásról a többi felé
        let mut equalities: Vec<Equality> = vec![];
        for (_v, occs) in var_pos {
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
        // 2) Literálok: minden atom-argumentumról külön equality (bal = a hely)
        for (occ, lit) in atom_pos {
            equalities.push(Equality {
                left: NodeArgRef { node: occ.node, arg: occ.arg },
                right: RightSide::Atom(lit),
            });
        }

        // Children csak name+arity
        let children_min = children.iter()
            .map(|(n, a, _)| ChildPred { name: n.clone(), arity: *a })
            .collect::<Vec<_>>();

        let out_clause = OutClause {
            children: children_min,
            equalities,
        };

        map.entry((hname.clone(), harity))
            .and_modify(|op| op.clauses.push(out_clause.clone()))
            .or_insert_with(|| OutPredicate {
                name: hname,
                arity: harity,
                clauses: vec![out_clause],
            });
    }

    let mut predicates = map.into_values().collect::<Vec<_>>();
    // stabil sorrend érdekében
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
