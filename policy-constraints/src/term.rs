//! Field encoding of ground Prolog terms and their Poseidon digests.
//!
//! A term is a tree of nodes. Every node has a tag, a payload, an arity and
//! child nodes, and its digest is
//!
//! ```text
//! digest = Poseidon(DOMAIN, tag, payload, arity, child_0, …, child_{K-1})
//! ```
//!
//! where `K = MAX_ARITY` and the digests of missing children are zero.
//!
//! | node       | tag      | payload         | children     |
//! |------------|----------|-----------------|--------------|
//! | atom `a`   | Atom     | `symbol("a")`   | none         |
//! | integer n  | Integer  | `integer(n)`    | none         |
//! | `[]`       | Nil      | 0               | none         |
//! | `[H\|T]`   | Cons     | 0               | `H`, `T`     |
//! | `f(A, …)`  | Compound | `symbol("f")`   | the arguments |
//!
//! Lists are cons cells, so every step of a path is a child index. A goal is
//! encoded as the term it is written as: `p(1, a)` is the compound `p/2`, and
//! a zero-arity goal is an atom.

use ff::{Field, FromUniformBytes, PrimeField};
use halo2_gadgets::poseidon::primitives::{ConstantLength, Hash, P128Pow5T3};
use halo2_proofs::pasta::Fp;
use policy_ir::Term;     

use crate::error::ModelError;

/// The largest arity of any goal or compound term.
pub const MAX_ARITY: usize = 4;

const NODE_HASH_INPUTS: usize = 4 + MAX_ARITY;
/// Domain separator of node digests (ASCII `BMNODE` + version 1).
const NODE_DOMAIN: u64 = 0x424d_4e4f_4445_0001;
const SYMBOL_DOMAIN: &[u8] = b"batchman-zkp/policy-constraints/symbol/v1";

/// The kind of a node. `Absent` marks a position that does not exist in the
/// term (for example the third argument of a binary compound); all cells of
/// an absent node are zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TermTag {
    Absent = 0,
    Atom = 1,
    Integer = 2,
    Nil = 3,
    Cons = 4,
    Compound = 5,
}

impl TermTag {
    pub fn field(self) -> Fp {
        Fp::from(self as u64)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Atom => "atom",
            Self::Integer => "integer",
            Self::Nil => "nil",
            Self::Cons => "cons",
            Self::Compound => "compound",
        }
    }

    /// The tag whose [`field`](Self::field) encoding is `value`, if any.
    pub fn from_field(value: Fp) -> Option<Self> {
        [
            Self::Absent,
            Self::Atom,
            Self::Integer,
            Self::Nil,
            Self::Cons,
            Self::Compound,
        ]
        .into_iter()
        .find(|tag| tag.field() == value)
    }

    /// The smallest and largest arity of a present node with this tag.
    pub fn arities(self) -> (usize, usize) {
        match self {
            Self::Absent | Self::Atom | Self::Integer | Self::Nil => (0, 0),
            Self::Cons => (2, 2),
            Self::Compound => (1, MAX_ARITY),
        }
    }
}

/// Encodes an atom or functor name as a field element.
pub fn symbol(name: &str) -> Fp {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SYMBOL_DOMAIN);
    hasher.update(&(name.len() as u64).to_le_bytes());
    hasher.update(name.as_bytes());
    let mut uniform = [0_u8; 64];
    hasher.finalize_xof().fill(&mut uniform);
    Fp::from_uniform_bytes(&uniform)
}

/// Encodes a signed integer as a field element (negative values as `p - |n|`).
pub fn integer(value: i64) -> Fp {
    let magnitude = Fp::from(value.unsigned_abs());
    if value < 0 { -magnitude } else { magnitude }
}

/// Digest of a present node. The caller supplies the child digests padded
/// with zeros to `MAX_ARITY`.
pub fn node_digest(tag: TermTag, payload: Fp, arity: usize, children: &[Fp; MAX_ARITY]) -> Fp {
    debug_assert!(tag != TermTag::Absent, "absent nodes have no digest");
    hash_node(tag.field(), payload, Fp::from(arity as u64), children)
}

/// [`node_digest`] over the cell values of a node, as the circuit sees them:
/// the tag and the arity are field elements, not checked to be meaningful.
pub fn hash_node(tag: Fp, payload: Fp, arity: Fp, children: &[Fp; MAX_ARITY]) -> Fp {
    let mut input = [Fp::ZERO; NODE_HASH_INPUTS];
    input[0] = Fp::from(NODE_DOMAIN);
    input[1] = tag;
    input[2] = payload;
    input[3] = arity;
    input[4..].copy_from_slice(children);
    Hash::<Fp, P128Pow5T3, ConstantLength<NODE_HASH_INPUTS>, 3, 2>::init().hash(input)
}

/// A variable-free term with the digest of every node precomputed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroundTerm {
    tag: TermTag,
    payload: Fp,
    children: Vec<GroundTerm>,
    digest: Fp,
}

impl GroundTerm {
    pub fn atom(name: &str) -> Self {
        Self::node(TermTag::Atom, symbol(name), Vec::new())
    }

    pub fn integer(value: i64) -> Self {
        Self::node(TermTag::Integer, integer(value), Vec::new())
    }

    pub fn nil() -> Self {
        Self::node(TermTag::Nil, Fp::ZERO, Vec::new())
    }

    pub fn cons(head: GroundTerm, tail: GroundTerm) -> Self {
        Self::node(TermTag::Cons, Fp::ZERO, vec![head, tail])
    }

    pub fn compound(functor: &str, args: Vec<GroundTerm>) -> Result<Self, ModelError> {
        if args.is_empty() {
            return Err(ModelError::new(format!(
                "compound {functor}() has no arguments; write the atom {functor} instead"
            )));
        }
        if args.len() > MAX_ARITY {
            return Err(ModelError::new(format!(
                "{functor}/{} exceeds the maximum arity {MAX_ARITY}",
                args.len()
            )));
        }
        Ok(Self::node(TermTag::Compound, symbol(functor), args))
    }

    /// Converts a parsed term. Fails on variables and on terms wider or
    /// larger than the model supports.
    pub fn from_term(term: &Term) -> Result<Self, ModelError> {
        match term {
            Term::Variable { .. } => Err(ModelError::new("term is not ground")),
            Term::Atom { value } => Ok(Self::atom(value)),
            Term::Integer { decimal } => decimal
                .parse::<i64>()
                .map(Self::integer)
                .map_err(|_| ModelError::new(format!("integer {decimal} does not fit in 64 bits"))),
            Term::Compound { functor, args } => {
                let args = args.iter().map(Self::from_term).collect::<Result<_, _>>()?;
                Self::compound(functor, args)
            }
            Term::List { items, tail } => {
                let mut list = match tail {
                    Some(tail) => Self::from_term(tail)?,
                    None => Self::nil(),
                };
                for item in items.iter().rev() {
                    list = Self::cons(Self::from_term(item)?, list);
                }
                Ok(list)
            }
        }
    }

    fn node(tag: TermTag, payload: Fp, children: Vec<GroundTerm>) -> Self {
        let mut child_digests = [Fp::ZERO; MAX_ARITY];
        for (digest, child) in child_digests.iter_mut().zip(&children) {
            *digest = child.digest;
        }
        let digest = node_digest(tag, payload, children.len(), &child_digests);
        Self {
            tag,
            payload,
            children,
            digest,
        }
    }

    pub fn tag(&self) -> TermTag {
        self.tag
    }

    pub fn payload(&self) -> Fp {
        self.payload
    }

    pub fn arity(&self) -> usize {
        self.children.len()
    }

    pub fn children(&self) -> &[GroundTerm] {
        &self.children
    }

    pub fn digest(&self) -> Fp {
        self.digest
    }

    /// The value of an integer node, `None` for every other node.
    pub fn integer_value(&self) -> Option<i64> {
        if self.tag != TermTag::Integer {
            return None;
        }
        let small = |value: Fp| -> Option<u64> {
            let repr = value.to_repr();
            let (low, high) = repr.as_ref().split_at(8);
            high.iter()
                .all(|byte| *byte == 0)
                .then(|| u64::from_le_bytes(low.try_into().expect("8 bytes")))
        };
        match small(self.payload) {
            Some(magnitude) => i64::try_from(magnitude).ok(),
            None => 0_i64.checked_sub_unsigned(small(-self.payload)?),
        }
    }

    /// The subterm reached by following child indices, if it exists.
    pub fn at(&self, path: &[u8]) -> Option<&GroundTerm> {
        path.iter()
            .try_fold(self, |node, &index| node.children.get(usize::from(index)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_term_types_have_distinct_digests() {
        let digest = |term: Term| GroundTerm::from_term(&term).unwrap().digest();
        let one = Term::Integer {
            decimal: "1".to_owned(),
        };
        let nil = Term::List {
            items: Vec::new(),
            tail: None,
        };
        assert_ne!(digest(Term::atom("1")), digest(one));
        assert_ne!(digest(Term::atom("[]")), digest(nil));
    }

    #[test]
    fn integer_values_round_trip() {
        for value in [0, 1, -1, 2000, -941_220, i64::MAX, i64::MIN] {
            assert_eq!(GroundTerm::integer(value).integer_value(), Some(value));
        }
        assert_eq!(GroundTerm::atom("a").integer_value(), None);
    }

    #[test]
    fn lists_are_cons_cells() {
        let list = GroundTerm::from_term(&Term::List {
            items: vec![Term::atom("a"), Term::atom("b")],
            tail: None,
        })
        .unwrap();
        let expected = GroundTerm::cons(
            GroundTerm::atom("a"),
            GroundTerm::cons(GroundTerm::atom("b"), GroundTerm::nil()),
        );
        assert_eq!(list, expected);
        assert_eq!(list.at(&[1, 0]), Some(&GroundTerm::atom("b")));
        assert_eq!(list.at(&[1, 1, 0]), None);
    }

    #[test]
    fn digest_depends_on_argument_order_and_functor() {
        let f = |name: &str, a, b| {
            GroundTerm::compound(name, vec![GroundTerm::integer(a), GroundTerm::integer(b)])
                .unwrap()
                .digest()
        };
        assert_ne!(f("f", 1, 2), f("f", 2, 1));
        assert_ne!(f("f", 1, 2), f("g", 1, 2));
    }

    #[test]
    fn negative_integers_are_field_negations() {
        assert_eq!(integer(-5) + integer(5), Fp::ZERO);
        assert_ne!(
            GroundTerm::integer(-1).digest(),
            GroundTerm::integer(1).digest()
        );
    }

    #[test]
    fn unsupported_terms_are_rejected() {
        let too_wide = Term::compound("f", vec![Term::atom("a"); MAX_ARITY + 1]);
        assert!(GroundTerm::from_term(&too_wide).is_err());
        assert!(GroundTerm::from_term(&Term::Variable { id: 0 }).is_err());
        assert!(
            GroundTerm::from_term(&Term::Integer {
                decimal: "99999999999999999999".to_owned()
            })
            .is_err()
        );
    }
}
