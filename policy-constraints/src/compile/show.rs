//! Prolog-like text of terms, for row descriptions and error messages.

use policy_ir::Term;

pub(crate) fn show(term: &Term) -> String {
    match term {
        Term::Variable { id } => format!("_V{id}"),
        Term::Atom { value } => quote_atom(value),
        Term::Integer { decimal } => decimal.clone(),
        Term::Compound { functor, args } => {
            let args: Vec<String> = args.iter().map(show).collect();
            format!("{}({})", quote_atom(functor), args.join(","))
        }
        Term::List { items, tail } => {
            let items: Vec<String> = items.iter().map(show).collect();
            match tail {
                Some(tail) => format!("[{}|{}]", items.join(","), show(tail)),
                None => format!("[{}]", items.join(",")),
            }
        }
    }
}

fn quote_atom(atom: &str) -> String {
    let alphanumeric = atom.starts_with(|c: char| c.is_ascii_lowercase())
        && atom.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    let symbolic = !atom.is_empty() && atom.chars().all(|c| "#$&*+-./:<=>?@^~\\".contains(c));
    if alphanumeric || symbolic || matches!(atom, "!" | ";" | ",") {
        atom.to_owned()
    } else {
        format!("'{}'", atom.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}
