use antlr4rust::{common_token_stream::CommonTokenStream, InputStream};
use antlr4rust::tree::{ParseTree, ParseTreeVisitorCompat};
use std::collections::{HashMap, HashSet};

pub mod parser;
use parser::prologlexer::prologLexer;
use parser::prologparser::*;
use parser::prologvisitor::prologVisitor;
use serde::Serialize;
use anyhow::*;
use std::fs;
use std::path::Path;

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
    // FONTOS: a children CSAK a BODY listákat tartalmazza (a head nincs benne),
    // de az indexelés LOGIKAI: head = children_node_list 0, a body sorok 1..N.
    children: Vec<Vec<ChildPred>>,
    equalities: Vec<Equality>,
}

#[derive(Serialize, Clone)]
struct ChildPred {
    name: String,
    arity: usize,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeArgRef4 {
    children_node_list: usize, // LOGIKAI index: 0 = head, 1.. = N. body-sor
    predicate: usize,          // az adott listában hányadik predikátum
    arg: usize,                // az adott predikátum argument indexe
    list_index: usize,         // tuple/list flatten index (0..k-1), tail = k, sima = 0
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
enum RightSide {
    Ref(NodeArgRef4),
    Atom(String),
}

#[derive(Serialize, Clone)]
struct Equality {
    left: NodeArgRef4,
    right: RightSide,
}


// ------------------ Belső (egyszerű) AST ------------------

#[derive(Debug, Clone)]
enum Term {
    Atom(String),
    Var(String),
    Predicate { name: String, args: Vec<Term> },
    ListCell { head: Box<Term>, tail: Box<Term> },
    EmptyList,
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
    head: Term,
    body: Vec<Term>, // a body vesszővel tagolt elemei. Minden elem egy "sor".
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

    // Egyszerű szögletes listaparszoló → cons-lánc
    fn parse_list(&self, s: &str) -> Term {
        let inner = &s[1..s.len() - 1];
        if inner.trim().is_empty() {
            return Term::EmptyList;
        }
        let (left, tail_opt) = if let Some(pos) = inner.find('|') {
            (inner[..pos].trim(), Some(inner[pos + 1..].trim().to_string()))
        } else {
            (inner.trim(), None)
        };

        let parts = split_top_level_commas(left)
            .into_iter()
            .map(|p| p.trim().to_string())
            .collect::<Vec<_>>();

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
            List_termContext(l) => Term::Atom(l.get_text()),
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

// Tuple "(a,b,c)" → mezők (underscore marad indexeléshez, de nem veszünk fel előfordulást)
fn parse_tuple_fields_keep_all(atom_str: &str) -> Vec<String> {
    let s = atom_str.trim();
    if !(s.starts_with('(') && s.ends_with(')')) {
        return vec![s.to_string()];
    }
    let inner = &s[1..s.len() - 1];
    split_top_level_commas(inner)
        .into_iter()
        .map(|p| p.trim().to_string())
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
            let args_s = args.iter().map(|a| term_as_string(a)).collect::<Vec<_>>().join(", ");
            format!("{}({})", name, args_s)
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

// ------------------ Builtin op-készlet ------------------

fn builtin_ops_set() -> HashSet<&'static str> {
    [
        "is", "+", "-", "*", "/", "div", "mod", "//", "rem", "**", "^", "<<", ">>",
        "<", ">", ">=", "=<", "=:=", "=\\=",
        "=", "\\=", "==", "\\==", "=..",
        "@<", "@>", "@>=", "@=<",
        "->", ",", ";",
        "\\+",
    ].into_iter().collect()
}

// Arith/rel kifejezésben láncolandó op-ok (az "is" kivételével)
fn chainable_op(name: &str) -> bool {
    matches!(name,
        "+" | "-" | "*" | "/" | "div" | "mod" | "//" | "rem" | "**" | "^" |
        "<<" | ">>" | "<" | ">" | ">=" | "=<" | "=:=" | "=\\=" | "=" | "\\=" | "==" | "\\=="
    )
}

// Inorder bejárás: bal → op → jobb; minden csomópontról (op,right) visszaadjuk
fn collect_right_args_inorder(term: &Term, acc: &mut Vec<(String, Term)>) {
    if let Term::Predicate { name, args } = term {
        if args.len() == 2 && chainable_op(name) {
            collect_right_args_inorder(&args[0], acc);
            acc.push((name.clone(), args[1].clone())); // op, right
            collect_right_args_inorder(&args[1], acc);
        }
    }
}

// Bal szélső levél kinyerése egy op-fából (ha van)
fn leftmost_leaf(term: &Term) -> Option<Term> {
    let mut cur = term.clone();
    loop {
        match cur {
            Term::Predicate { ref name, ref args } if args.len() == 2 && chainable_op(name) => {
                cur = args[0].clone();
            }
            _ => return Some(cur),
        }
    }
}

// ------------------ Clause → kimenet ------------------

fn to_output(clauses: Vec<Clause>) -> OutRoot {
    let mut map: HashMap<(String, usize), OutPredicate> = HashMap::new();
    let builtin_ops = builtin_ops_set();

    // --------- Egy előfordulás (4D ref) ---------
    #[derive(Clone, Copy, Debug)]
    struct Occ4 { l: usize, p: usize, a: usize, li: usize }

    // Flatten egy argumentumról: list/tuple saját szabály szerint
    fn flatten_arg_collect(
        term: &Term,
        l: usize, p: usize, a: usize,
        var_pos: &mut HashMap<String, Vec<Occ4>>,
        atom_pos: &mut Vec<(Occ4, String)>,
    ) {
        match term {
            Term::ListCell { head, tail } => {
                let head_str = match &**head {
                    Term::Atom(s) => s.clone(),
                    Term::Var(v) => v.clone(),
                    other => term_as_string(other),
                };
                let fields = parse_tuple_fields_keep_all(&head_str);
                let mut idx = 0usize;
                for f in fields {
                    if f != "_" {
                        if is_var_name(&f) {
                            var_pos.entry(f.clone()).or_default().push(Occ4 { l, p, a, li: idx });
                        } else {
                            atom_pos.push((Occ4 { l, p, a, li: idx }, f.clone()));
                        }
                    }
                    idx += 1;
                }
                // Tail = k, nem bontjuk tovább
                let tail_str = match &**tail {
                    Term::Var(v) => v.clone(),
                    other => term_as_string(other),
                };
                if tail_str != "_" {
                    if is_var_name(&tail_str) {
                        var_pos.entry(tail_str.clone()).or_default().push(Occ4 { l, p, a, li: idx });
                    } else {
                        atom_pos.push((Occ4 { l, p, a, li: idx }, tail_str));
                    }
                }
            }
            Term::Var(v) => {
                // minden '_' és '_Valami' wildcard → IGNORE
                if v == "_" || v.starts_with("_") {
                    return;
                }
                var_pos.entry(v.clone()).or_default().push(Occ4 { l, p, a, li: 0 });
            }
            Term::Atom(s) => {
                atom_pos.push((Occ4 { l, p, a, li: 0 }, s.clone()));
            }
            // Beágyazott predikátumot itt nem bontunk (kivéve a láncolt op-oknál külön kezeljük)
            _ => {}
        }
    }

    for cl in clauses {
        // --- Head
        let (hname, hargs) = match &cl.head {
            Term::Predicate { name, args } => (name.clone(), args.clone()),
            Term::Atom(a) => (a.clone(), vec![]),
            Term::Var(v) => (v.clone(), vec![]),
            _ => ("".to_string(), vec![]),
        };
        let harity = hargs.len();

        // Equalities gyűjtők
        let mut var_pos: HashMap<String, Vec<Occ4>> = HashMap::new();
        let mut atom_pos: Vec<(Occ4, String)> = vec![];

        // Default clause detektálás (pl consumptionClass default)
        let mut is_default_clause = false;
        if cl.body.len() == 1 {
            if let Term::Predicate { name, args:_ } = &cl.body[0] {
                if name == "=" {
                    // consumptionClass(_RollingConsumptionVar,Class):- Class='low'.
                    is_default_clause = true;
                }
            }
        }

        // LOGIKAI head (l=0): head argumentumok előfordulásai
        for (ai, a) in hargs.iter().enumerate() {
            flatten_arg_collect(a, /*l*/0, /*p*/0, ai, &mut var_pos, &mut atom_pos);
        }

        // children: csak BODY-listák
        let mut children_lists: Vec<Vec<ChildPred>> = vec![];

        // ---- BODY: minden “sor” külön predicate-list
        // LOGIKAI l = 1 + index; a children-ben ez a (l-1). elem
        for (list_idx_from0, t) in cl.body.iter().enumerate() {
            let l = 1 + list_idx_from0; // LOGIKAI lista index
            let mut this_list: Vec<ChildPred> = vec![];

            match t {
                Term::Predicate { name, args } if name.as_str() == "=" => {
        let left_str = term_as_string(&args[0]);
        let right_str = term_as_string(&args[1]);

        // ------------------------------
        // Ha a két oldal tökéletesen azonos atom → implicit match
        // → NEM kell gyermek node ("=" nem kerül children-be)
        // ------------------------------
        if is_default_clause || (left_str == right_str && !is_var_name(&left_str)) {
            flatten_arg_collect(&args[0],  l, 0, 0, &mut var_pos, &mut atom_pos);
            flatten_arg_collect(&args[1],  l, 0, 0, &mut var_pos, &mut atom_pos);
            // NINCS push this_list
            continue;
        }

        // ------------------------------
        // Ha a két oldal nem atom–atom egyenlőség
        // VAGY explicit levezetett egyenlőség (pl mid=mid savingsClass végén)
        // → KELL child "="
        // ------------------------------
        this_list.push(ChildPred { name: name.clone(), arity: 2 });
        flatten_arg_collect(&args[0], l, 0, 0, &mut var_pos, &mut atom_pos);
        flatten_arg_collect(&args[1], l, 0, 1, &mut var_pos, &mut atom_pos);
        children_lists.push(this_list);
        continue;
    }

                Term::Predicate { name, args } if builtin_ops.contains(name.as_str()) => {
                    // Builtin: első op 2-operandusú, majd RHS inorder lánc opjai 1-operandusúak
                    let first_name = name.clone();
                    let (left, right) = if args.len() == 2 {
                        (args[0].clone(), args[1].clone())
                    } else if args.len() == 1 {
                        (Term::Atom("".into()), args[0].clone())
                    } else {
                        (Term::Atom("".into()), Term::Atom("".into()))
                    };
                    this_list.push(ChildPred { name: first_name.clone(), arity: 2 });

                    // első op bal arg
                    flatten_arg_collect(&left,  l, 0, 0, &mut var_pos, &mut atom_pos);

                    // ha a right nem op-fa, közvetlenül az első op jobb argjához
                    let mut right_is_chain_root = false;
                    if let Term::Predicate { name: rname, args: rargs } = &right {
                        if rargs.len() == 2 && chainable_op(rname) {
                            right_is_chain_root = true;
                        }
                    }
                    if !right_is_chain_root {
                        flatten_arg_collect(&right, l, 0, 1, &mut var_pos, &mut atom_pos);
                    } else {
                        // <<< ÚJ >>> Ha op-fa, akkor a BAL SZÉLSŐ LEVÉL is legyen felvéve az 'is' jobb arg (l,0,1) helyére,
                        // hogy a head-beli azonos nevű változóval equality jöjjön létre.
                        if let Some(leftmost) = leftmost_leaf(&right) {
                            flatten_arg_collect(&leftmost, l, 0, 1, &mut var_pos, &mut atom_pos);
                        }
                    }

                    // Jobb oldal lánc opjai balról jobbra (mind arity=1), és CSAK a jobboldali operandust gyűjtjük (arg=0)
                    let mut chain: Vec<(String, Term)> = vec![];
                    collect_right_args_inorder(&right, &mut chain);
                    for (k, (opn, rterm)) in chain.into_iter().enumerate() {
                        let pred_idx = 1 + k; // ebben a listában az első op után
                        this_list.push(ChildPred { name: opn, arity: 1 });
                        flatten_arg_collect(&rterm, l, pred_idx, 0, &mut var_pos, &mut atom_pos);
                    }
                }

                Term::Predicate { name, args } => {
                    // Normál predikátum: 1 elemű lista
                    this_list.push(ChildPred { name: name.clone(), arity: args.len() });
                    for (ai, a) in args.iter().enumerate() {
                        flatten_arg_collect(a, l, 0, ai, &mut var_pos, &mut atom_pos);
                    }
                }

                _ => {
                    // Üres lista: nincs pred (pl. “true” szerű elem). Hozzáadjuk üresen.
                }
            }

            children_lists.push(this_list);
        }

        // --- Equalities összeállítása (változók láncolása + literálok)
        let mut equalities: Vec<Equality> = vec![];
       if is_default_clause {
    // csak head-ben levő egyenlőségek megtartása

    let mut filtered_equalities = vec![];

    for occs in var_pos.values() {
        if occs.len() > 1 {
            let base = occs[0];
            if base.l == 0 {  // HEAD-ben volt
                for &o in occs.iter().skip(1) {
                    if o.l == 0 {  // ő is HEAD-ben
                        add_ref_eq(&mut filtered_equalities, &mut HashSet::new(), base, o);
                    }
                }
            }
        }
    }

   for (o, lit) in &atom_pos {
        if !lit.starts_with("_") {
            add_atom_eq(&mut filtered_equalities, &mut HashSet::new(), *o, lit.clone());
        }
    }

    let out_clause = OutClause {
        children: vec![],                    // default: nincs subtree
        equalities: filtered_equalities,     // csak HEAD equalities
    };

    map.entry((hname.clone(), harity))
        .and_modify(|op| op.clauses.push(out_clause.clone()))
        .or_insert_with(|| OutPredicate {
            name: hname,
            arity: harity,
            clauses: vec![out_clause],
        });

    continue;
}

        let mut seen_refs: HashSet<(NodeArgRef4, NodeArgRef4)> = HashSet::new();
        let mut seen_atoms: HashSet<(NodeArgRef4, String)> = HashSet::new();

        fn add_ref_eq(
            equalities: &mut Vec<Equality>,
            seen_refs: &mut HashSet<(NodeArgRef4, NodeArgRef4)>,
            l: Occ4, r: Occ4
        ) {
            let a = NodeArgRef4 { children_node_list: l.l, predicate: l.p, arg: l.a, list_index: l.li };
            let b = NodeArgRef4 { children_node_list: r.l, predicate: r.p, arg: r.a, list_index: r.li };
            if a == b { return; }
            let key = if (a.children_node_list, a.predicate, a.arg, a.list_index)
                   <= (b.children_node_list, b.predicate, b.arg, b.list_index) { (a,b) } else { (b,a) };
            if seen_refs.insert(key) {
                equalities.push(Equality { left: key.0, right: RightSide::Ref(key.1) });
            }
        }

        fn add_atom_eq(
            equalities: &mut Vec<Equality>,
            seen_atoms: &mut HashSet<(NodeArgRef4, String)>,
            l: Occ4, lit: String
        ) {
            let a = NodeArgRef4 { children_node_list: l.l, predicate: l.p, arg: l.a, list_index: l.li };
            if seen_atoms.insert((a, lit.clone())) {
                equalities.push(Equality { left: a, right: RightSide::Atom(lit) });
            }
        }

        // Változók láncolása
        for occs in var_pos.values() {
            if occs.len() > 1 {
                let base = occs[0];
                for &o in occs.iter().skip(1) {
                    add_ref_eq(&mut equalities, &mut seen_refs, base, o);
                }
            }
        }
        // Literál kötés
        for (o, lit) in atom_pos {
            if !lit.starts_with("_") {
                add_atom_eq(&mut equalities, &mut seen_atoms, o, lit);
            }
        }

        let out_clause = OutClause {
            children: children_lists,
            equalities,
        };

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

    let dir = Path::new("input");
    let file_path = dir.join("rules.json");

    fs::create_dir_all(dir).expect("failed to create input directory");

    // JSON kiírás
    fs::write(&file_path, &json).expect("write failed");

    println!("JSON saved to {}", file_path.display());
}
