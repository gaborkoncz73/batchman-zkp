use anyhow::Result;
use common::{
    circuit_dialect::CircuitLimits, data::FactEntry, utils_2::common_helpers::MAX_FACTS_HASHES,
};
use halo2curves::ff::PrimeField;
use num_bigint::BigUint;
use std::{fs, path::Path};

fn main() -> Result<()> {
    let config_file = "issue/src/facts.yaml";
    let file_content = fs::read_to_string(config_file)?;
    let fact_configs: Vec<FactEntry> = serde_yaml::from_str(&file_content)?;
    let limits: CircuitLimits =
        serde_json::from_str(&fs::read_to_string("config/circuit-limits.json")?)?;
    let facts = tokens::parse_fact_registry(&fact_configs, &limits, config_file)?;

    let mut fact_hashes: Vec<String> = facts
        .iter()
        .map(|f| {
            let hash_bytes = f.commitment()?.to_repr();
            let hash_int = BigUint::from_bytes_le(hash_bytes.as_ref());
            Ok(hash_int.to_str_radix(10))
        })
        .collect::<Result<_>>()?;

    while fact_hashes.len() < MAX_FACTS_HASHES {
        fact_hashes.push("0".to_string());
    }

    let out_path = Path::new("input/fact_hashes.json");
    fs::create_dir_all(out_path.parent().unwrap())?;
    let mut json = serde_json::to_string_pretty(&fact_hashes)?;
    json.push('\n');
    fs::write(out_path, json)?;

    println!("Saved {} fact hashes to {:?}", fact_hashes.len(), out_path);
    Ok(())
}
