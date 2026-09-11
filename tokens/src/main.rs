use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use common::circuit_dialect::CircuitLimits;
use tokens::lowering::{compile_policy_for_current_circuit, to_circuit_v1_json};
use tokens::{parse_policy, to_canonical_json};

const DEFAULT_SOURCE: &str = "prolog/policy.pl";
const DEFAULT_SEMANTIC_OUTPUT: &str = "input/rules.json";
const DEFAULT_LIMITS: &str = "config/circuit-limits.json";

fn main() -> Result<()> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments
        .iter()
        .any(|argument| argument == "-h" || argument == "--help")
    {
        print_usage();
        return Ok(());
    }
    if arguments.len() > 4 {
        bail!("too many arguments; use --help for usage");
    }

    let source_path = PathBuf::from(arguments.first().map_or(DEFAULT_SOURCE, String::as_str));
    let semantic_output = PathBuf::from(
        arguments
            .get(1)
            .map_or(DEFAULT_SEMANTIC_OUTPUT, String::as_str),
    );
    let compatibility_output = arguments.get(2).map(PathBuf::from);
    let limits_path = PathBuf::from(arguments.get(3).map_or(DEFAULT_LIMITS, String::as_str));

    reject_path_collisions(
        &source_path,
        &semantic_output,
        compatibility_output.as_deref(),
        &limits_path,
    )?;

    let source = fs::read_to_string(&source_path)
        .with_context(|| format!("failed to read {}", source_path.display()))?;
    let source_name = source_path.to_string_lossy().replace('\\', "/");
    let policy = parse_policy(&source, &source_name)?;
    let semantic_json = to_canonical_json(&policy)?;
    let compatibility_json = if compatibility_output.is_some() {
        let limits_text = fs::read_to_string(&limits_path)
            .with_context(|| format!("failed to read {}", limits_path.display()))?;
        let limits: CircuitLimits = serde_json::from_str(&limits_text)
            .with_context(|| format!("failed to parse {}", limits_path.display()))?;
        let compiled = compile_policy_for_current_circuit(&policy, &limits)?;
        Some(to_circuit_v1_json(compiled.layout())?)
    } else {
        None
    };

    // All parsing and checked compilation completes before either output is
    // touched, so a semantic/circuit validation error cannot leave a partial
    // update behind.
    write_atomic(&semantic_output, &semantic_json)?;
    println!(
        "Semantic policy IR written to {}",
        semantic_output.display()
    );
    if let (Some(compatibility_output), Some(circuit_json)) =
        (compatibility_output, compatibility_json)
    {
        write_atomic(&compatibility_output, &circuit_json)?;
        println!(
            "Optional circuit compatibility IR written to {}",
            compatibility_output.display()
        );
    }
    Ok(())
}

fn reject_path_collisions(
    source: &Path,
    semantic_output: &Path,
    compatibility_output: Option<&Path>,
    limits: &Path,
) -> Result<()> {
    let source = lexical_absolute(source)?;
    let semantic = lexical_absolute(semantic_output)?;
    if source == semantic {
        bail!("semantic output must not overwrite the Prolog source");
    }
    if let Some(compatibility_output) = compatibility_output {
        let compatibility = lexical_absolute(compatibility_output)?;
        if compatibility == source {
            bail!("compatibility output must not overwrite the Prolog source");
        }
        if compatibility == semantic {
            bail!("semantic and compatibility outputs must use different paths");
        }
        let limits = lexical_absolute(limits)?;
        if compatibility == limits || semantic == limits {
            bail!("generated output must not overwrite the circuit limits file");
        }
    }
    Ok(())
}

fn lexical_absolute(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        use std::path::Component;
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let file_name = path
        .file_name()
        .context("output path has no file name")?
        .to_string_lossy();
    let temporary = parent.join(format!(".{file_name}.tmp"));
    fs::write(&temporary, content)
        .with_context(|| format!("failed to write {}", temporary.display()))?;
    fs::rename(&temporary, path).with_context(|| {
        format!(
            "failed to atomically replace {} with {}",
            path.display(),
            temporary.display()
        )
    })?;
    Ok(())
}

fn print_usage() {
    println!(
        "Usage: tokens [SOURCE.pl [RULES.json [OPTIONAL_COMPATIBILITY_RULES.json [LIMITS.json]]]]\n\
         Defaults:\n  source: {DEFAULT_SOURCE}\n  semantic output: {DEFAULT_SEMANTIC_OUTPUT}\n\
         limits: {DEFAULT_LIMITS}\n\
         The third argument is an optional debug/golden projection; prove and verify do not read it."
    );
}
