use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use policy_parser::{parse_policy, to_canonical_json};

const DEFAULT_SOURCE: &str = "prolog/policy.pl";
const DEFAULT_OUTPUT: &str = "input/rules.json";

fn main() -> Result<()> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments
        .iter()
        .any(|argument| argument == "-h" || argument == "--help")
    {
        print_usage();
        return Ok(());
    }
    if arguments.len() > 2 {
        bail!("too many arguments; use --help for usage");
    }

    let source_path = PathBuf::from(arguments.first().map_or(DEFAULT_SOURCE, String::as_str));
    let output_path = PathBuf::from(arguments.get(1).map_or(DEFAULT_OUTPUT, String::as_str));
    if lexical_absolute(&source_path)? == lexical_absolute(&output_path)? {
        bail!("semantic output must not overwrite the Prolog source");
    }

    let source = fs::read_to_string(&source_path)
        .with_context(|| format!("failed to read {}", source_path.display()))?;
    let source_name = source_path.to_string_lossy().replace('\\', "/");
    // Parsing and validation complete before the output is touched, so an
    // error cannot leave a partial update behind.
    let json = to_canonical_json(&parse_policy(&source, &source_name)?)?;
    write_atomic(&output_path, &json)?;
    println!("Semantic policy IR written to {}", output_path.display());
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
        "Usage: policy-parser [SOURCE.pl [RULES.json]]\n\
         Defaults:\n  source: {DEFAULT_SOURCE}\n  output: {DEFAULT_OUTPUT}"
    );
}
