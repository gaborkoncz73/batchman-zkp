//! Compatibility re-exports for callers that previously imported lowering
//! from the parser crate.  The parser-independent compiler lives in `common`
//! so the prover and verifier share exactly one implementation.

pub use common::policy_compiler::{
    CompiledPolicy, compile_policy_for_current_circuit, to_circuit_v1_json,
};
