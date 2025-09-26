use std::{fs::File, io::BufReader};

use ark_bn254::{Bn254, Fr};
use ark_groth16::{ Groth16, VerifyingKey};
use ark_snark::SNARK;
use serde::{ de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use base64::{engine::general_purpose, Engine as _};
use serde_json::from_reader;
use anyhow::Result;

#[derive(CanonicalDeserialize, CanonicalSerialize)]
pub struct VerifyInput {
    pub vk: VerifyingKey<Bn254>,
    pub public_inputs: Vec<Fr>,
    pub proof: <Groth16<Bn254> as SNARK<Fr>>::Proof,
}

impl Serialize for VerifyInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error as _;
        let mut v = Vec::new();
        self.serialize_uncompressed(&mut v)
            .map_err(|e| S::Error::custom(e.to_string()))?;
        serializer.serialize_str(&general_purpose::STANDARD.encode(&v))
    }
}

impl<'de> Deserialize<'de> for VerifyInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;
        let v = String::deserialize(deserializer)?;
        let decoded = general_purpose::STANDARD
            .decode(v)
            .map_err(|e| D::Error::custom(e.to_string()))?;
        Self::deserialize_uncompressed(&decoded[..]).map_err(|e|D::Error::custom(e.to_string()))
    }
}

pub fn load_input<T: DeserializeOwned>(path: &str) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(from_reader(reader)?)
}