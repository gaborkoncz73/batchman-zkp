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

#[derive(Serialize, Clone, Eq, PartialEq, Hash)]
struct NodeArgRef {
    node: usize,       // 0 = head, 1.. = body pred index + 1
    arg: usize,        // argument index within that node
    list_index: usize, // 0 = not in flattened list; 1.. = flattened tuple field index; tail = last (k+1)
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
enum RightSide {
    Ref(NodeArgRef),
    Atom(String), // minden literal stringként marad (számok is)
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
#[serde(tag = "kind")]
enum Operand {
    Ref { node: usize, arg: usize },
    Atom { value: String },
    Var { name: String },
    BuiltIn { operator: String, operands: Vec<Operand> },
}

// ------------------ Belső (egyszerű) AST ------------------

#[derive(Debug, Clone)]
enum Term {
    Atom(String),
    Var(String),
    Predicate { name: String, args: Vec<Term> },
    ListCell { head: Box<Term>, tail: Box<Term> },
    EmptyList, // []
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

    // Egyszerű szögletes listaparszoló → cons-lánc (ListCell)
    fn parse_list(&self, s: &str) -> Term {
        let inner = &s[1..s.len() - 1];
        if inner.trim().is_empty() {
            return Term::EmptyList;
        }
        // vágjuk le a '|' utáni tailt (ha van), külön stringként
        let (left, tail_opt) = if let Some(pos) = inner.find('|') {
            (inner[..pos].trim(), Some(inner[pos + 1..].trim().to_string()))
        } else {
            (inner.trim(), None)
        };

        // bal oldal felső szintű vesszők mentén
        let parts = split_top_level_commas(left)
            .into_iter()
            .map(|p| p.trim().to_string())
            .collect::<Vec<_>>();

        // építsük vissza konsz-lánccá
        let mut t = match tail_opt {
            Some(tail_s) => self.parse_term_str(&tail_s),
            None => Term::EmptyList,
        };
        for elem in parts.into_iter().rev() {
            t = Term::ListCell {
                head: Box::new(self.parse_term_str(&elem)),
                tail: Box::new(t),
            };
        }
        t
    }

    fn parse_term_str(&self, s: &str) -> Term {
        if s.starts_with('[') && s.ends_with(']') {
            return self.parse_list(s);
        }
        // tuple / atom / var mind Atom-ként (string) marad – később stringből bontunk tuple-t
        Term::Atom(s.into())
    }

    fn parse_term(&self, ctx: &TermContextAll) -> Term {
        use TermContextAll::*;
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
                // már fent kezeltük → itt fallback
                Term::Atom(l.get_text())
            }
            Compound_termContext(c) => {
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
                let left = self.parse_term(&b.term(0).unwrap());
                let right = self.parse_term(&b.term(1).unwrap());
                let op = b.operator_().unwrap().get_text();
                Term::Predicate { name: op, args: vec![left, right] }
            }
            Unary_operatorContext(u) => {
                let arg = self.parse_term(&u.term().unwrap());
                let op = u.operator_().unwrap().get_text();
                Term::Predicate { name: op, args: vec![arg] }
            }
            _ => Term::Atom(ctx.get_text()),
        }
    }
}

// Segéd: felső szintű vesszők szerinti split (zárójelek figyelembevétele)
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth_paren = 0;
    let mut last = 0;
    let chars: Vec<char> = s.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        match ch {
            '(' => depth_paren += 1,
            ')' => if depth_paren > 0 { depth_paren -= 1 },
            ',' if depth_paren == 0 => {
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

// Tuple "(a,b,c)" → mezők top-level splitje, whitespace-trim, '_' kiszűrés
fn parse_tuple_fields(atom_str: &str) -> Vec<String> {
    let s = atom_str.trim();
    if !(s.starts_with('(') && s.ends_with(')')) {
        return vec![s.to_string()];
    }
    let inner = &s[1..s.len() - 1];
    split_top_level_commas(inner)
        .into_iter()
        .map(|p| p.trim().to_string())
        .filter(|p| p != "_" ) // '_' ignorálása
        .collect()
}

fn is_var_name(s: &str) -> bool {
    if s == "_" || s.is_empty() { return false; }
    let first = s.chars().next().unwrap();
    first.is_ascii_uppercase() || first == '_'
}

fn term_as_string(t: &Term) -> String {
    match t {
        Term::Atom(s) => s.clone(),
        Term::Var(v) => v.clone(),
        Term::EmptyList => "[]".to_string(),
        Term::Predicate { name, args } => {
            if name == "." && args.len() == 2 {
                // cons-list to string best-effort
                let mut elems = vec![];
                let mut cur = t.clone();
                let mut guard = 0usize;
                while let Term::Predicate { name, args } = cur.clone() {
                    if name != "." || args.len() != 2 { break; }
                    elems.push(term_as_string(&args[0]));
                    cur = args[1].clone();
                    guard += 1; if guard > 1024 { break; }
                }
                let tail_str = match cur {
                    Term::EmptyList => "".to_string(),
                    other => format!("|{}", term_as_string(&other)),
                };
                let body = if !elems.is_empty() { elems.join(",") } else { "".into() };
                if tail_str.is_empty() {
                    format!("[{}]", body)
                } else {
                    format!("[{}{}]", body, tail_str)
                }
            } else {
                // generic term
                let args_s = args.iter().map(|a| term_as_string(a)).collect::<Vec<_>>().join(", ");
                format!("{}({})", name, args_s)
            }
        }
        Term::ListCell { head, tail } => {
            let mut elems = vec![term_as_string(head)];
            let mut cur = (*tail.clone());
            let mut guard = 0usize;
            while let Term::ListCell { head: h, tail: t2 } = cur.clone() {
                elems.push(term_as_string(&h));
                cur = (*t2).clone();
                guard += 1; if guard > 1024 { break; }
            }
            let tail_str = match cur {
                Term::EmptyList => "".to_string(),
                other => format!("|{}", term_as_string(&other)),
            };
            let body = elems.join(",");
            if tail_str.is_empty() { format!("[{}]", body) } else { format!("[{}{}]", body, tail_str) }
        }
    }
}

// ------------------ Visitor impl ------------------

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
    let mut map: HashMap<(String, usize), OutPredicate> = HashMap::new();

    let builtin_ops: HashSet<&'static str> = [
        "is", "+", "-", "*", "/", "div", "mod", "//", "rem", "**", "^", "<<", ">>",
        "<", ">", ">=", "=:=", "=\\=",
        "=", "\\=", "==", "\\==", "=..",
        "@<", "@>", "@>=",
        "->", ",", ";",
        "\\+",
    ].into_iter().collect();

    for cl in clauses {
        // Head
        let (hname, hargs) = match &cl.head {
            Term::Predicate { name, args } => (name.clone(), args.clone()),
            Term::Atom(a) => (a.clone(), vec![]),
            Term::Var(v) => (v.clone(), vec![]),
            _ => ("".to_string(), vec![]),
        };
        let harity = hargs.len();

        // Body → children vs builtins
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

        // --- Equalities (gyűjtés) ---
        #[derive(Copy, Clone)]
        struct Occ { node: usize, arg: usize, list_index: usize }

        let mut var_pos: HashMap<String, Vec<Occ>> = HashMap::new();
        let mut atom_pos: Vec<(Occ, String)> = vec![];

        // Head arg bejárás: speciális – ha [ (tuple) | Tail ] → a tuple-t flatteneljük (list_index = 1..), Tail a legutolsó (k+1) stringként.
        for (hi, a) in hargs.iter().enumerate() {
            match a {
                Term::ListCell { head, tail } => {
                    // csak az ELSŐ cell headje: ha Atom és "(…)" tuple, akkor flatten
                    let head_str = match &**head {
                        Term::Atom(s) => s.clone(),
                        Term::Var(v) => v.clone(),
                        other => term_as_string(other),
                    };
                    let fields = parse_tuple_fields(&head_str);
                    let mut list_idx = 1usize;
                    for f in fields {
                        if f == "_" { continue; }
                        if is_var_name(&f) {
                            var_pos.entry(f).or_default().push(Occ { node: 0, arg: hi, list_index: list_idx });
                        } else {
                            atom_pos.push((Occ { node: 0, arg: hi, list_index: list_idx }, f));
                        }
                        list_idx += 1;
                    }
                    // Tail → a legutolsó
                    let tail_str = match &**tail {
                        Term::Var(v) => v.clone(),
                        other => term_as_string(other),
                    };
                    // Tail lehet változó vagy literal/string
                    if is_var_name(&tail_str) && tail_str != "_" {
                        var_pos.entry(tail_str).or_default().push(Occ { node: 0, arg: hi, list_index: list_idx });
                    } else {
                        atom_pos.push((Occ { node: 0, arg: hi, list_index: list_idx }, tail_str));
                    }
                }
                Term::Var(v) => {
                    if v != "_" {
                        var_pos.entry(v.clone()).or_default().push(Occ { node: 0, arg: hi, list_index: 0 });
                    }
                }
                Term::Atom(s) => {
                    atom_pos.push((Occ { node: 0, arg: hi, list_index: 0 }, s.clone()));
                }
                _ => { /* más head forma: nem flatteneljük */ }
            }
        }

        // BODY: sima (list_index = 0)
        for (child_idx, (_nm, _ar, args)) in children.iter().enumerate() {
            let node_index = 1 + child_idx;
            for (ai, a) in args.iter().enumerate() {
                match a {
                    Term::Var(v) => {
                        if v != "_" {
                            var_pos.entry(v.clone()).or_default().push(Occ { node: node_index, arg: ai, list_index: 0 });
                        }
                    }
                    Term::Atom(s) => {
                        atom_pos.push((Occ { node: node_index, arg: ai, list_index: 0 }, s.clone()));
                    }
                    _ => { /* nem flatteneljük a body listákat */ }
                }
            }
        }

        // '=' literal binding (var = atom | atom = var) → list_index = 0
        for op in &builtin_terms {
            if let Term::Predicate { name, args } = op {
                if name == "=" && args.len() == 2 {
                    match (&args[0], &args[1]) {
                        (Term::Var(v), Term::Atom(lit)) | (Term::Atom(lit), Term::Var(v)) if v != "_" => {
                            // keressük a legelső előfordulását – mindegy, hogy hol
                            if let Some(occs) = var_pos.get(v) {
                                if let Some(first) = occs.first() {
                                    atom_pos.push((*first, lit.clone()));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Egyenletek építése (duplikáció és ön-egyenlőség nélkül)
        let mut equalities: Vec<Equality> = vec![];
        let mut seen: HashSet<String> = HashSet::new();

        fn push_ref_eq(seen: &mut HashSet<String>, equalities: &mut Vec<Equality>, l: Occ, r: Occ) {
            if l.node == r.node && l.arg == r.arg && l.list_index == r.list_index {
                return; // önmagára
            }
            let key = format!("R:{}:{}:{}->{}:{}:{}", l.node, l.arg, l.list_index, r.node, r.arg, r.list_index);
            if seen.insert(key) {
                equalities.push(Equality {
                    left: NodeArgRef { node: l.node, arg: l.arg, list_index: l.list_index },
                    right: RightSide::Ref(NodeArgRef { node: r.node, arg: r.arg, list_index: r.list_index }),
                });
            }
        }
        fn push_atom_eq(seen: &mut HashSet<String>, equalities: &mut Vec<Equality>, l: Occ, lit: String) {
            let key = format!("A:{}:{}:{}={}", l.node, l.arg, l.list_index, lit);
            if seen.insert(key) {
                equalities.push(Equality {
                    left: NodeArgRef { node: l.node, arg: l.arg, list_index: l.list_index },
                    right: RightSide::Atom(lit),
                });
            }
        }

        // Var linking
        for (_v, occs) in &var_pos {
            if occs.len() > 1 {
                let base = occs[0];
                for other in occs.iter().copied().skip(1) {
                    push_ref_eq(&mut seen, &mut equalities, base, other);
                }
            }
        }
        // Literal bindings (minden literal string)
        for (occ, lit) in atom_pos {
            push_atom_eq(&mut seen, &mut equalities, occ, lit);
        }

        // Children minimal
        let children_min = children.iter()
            .map(|(n, a, _)| ChildPred { name: n.clone(), arity: *a })
            .collect::<Vec<_>>();

        // --- Builtin operandusfa (változatlan a korábbi működéshez képest) ---
        let resolve_var_ref = |vname: &str| -> Option<(usize, usize)> {
            // csak node,arg szükséges a beépítettekhez; a list_index itt nem játszik
            var_pos.get(vname).and_then(|occs| occs.first().map(|o| (o.node, o.arg)))
        };

        fn term_to_operand(
            t: &Term,
            is_builtin: &dyn Fn(&str) -> bool,
            resolve: &dyn Fn(&str) -> Option<(usize, usize)>,
        ) -> Operand {
            match t {
                Term::Atom(s) => Operand::Atom { value: s.clone() },
                Term::Var(v) => {
                    if v == "_" {
                        Operand::Var { name: v.clone() }
                    } else if let Some((node, arg)) = resolve(v) {
                        Operand::Ref { node, arg }
                    } else {
                        Operand::Var { name: v.clone() }
                    }
                }
                Term::Predicate { name, args } => {
                    if is_builtin(name) {
                        let ops = args.iter().map(|a| term_to_operand(a, is_builtin, resolve)).collect();
                        Operand::BuiltIn { operator: name.clone(), operands: ops }
                    } else {
                        Operand::Atom { value: format!("{}(…)", name) }
                    }
                }
                Term::ListCell { .. } | Term::EmptyList => {
                    // beépített operandusfában listát nem bontunk
                    Operand::Atom { value: "[]_or_list".into() }
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
