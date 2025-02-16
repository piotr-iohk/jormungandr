use chain_crypto::{Blake2b256, Curve25519_2HashDH, Ed25519, PublicKey, SecretKey, SumEd25519_12};
use chain_impl_mockchain::leadership::{bft, BftLeader, GenesisLeader};
use serde::Deserialize;
use std::path::Path;

pub mod enclave;

/// hold the node's bft secret setting
#[derive(Clone, Deserialize)]
pub struct Bft {
    #[serde(deserialize_with = "jormungandr_utils::serde::crypto::deserialize_secret")]
    signing_key: bft::SigningKey,
}

/// the genesis praos setting
///
#[derive(Clone, Deserialize)]
pub struct GenesisPraos {
    #[serde(deserialize_with = "jormungandr_utils::serde::crypto::deserialize_hash")]
    node_id: Blake2b256,
    #[serde(deserialize_with = "jormungandr_utils::serde::crypto::deserialize_secret")]
    sig_key: SecretKey<SumEd25519_12>,
    #[serde(deserialize_with = "jormungandr_utils::serde::crypto::deserialize_secret")]
    vrf_key: SecretKey<Curve25519_2HashDH>,
}

/// the genesis praos setting
///
#[derive(Clone, Deserialize)]
pub struct GenesisPraosPublic {
    #[serde(deserialize_with = "jormungandr_utils::serde::crypto::deserialize_public")]
    sig_key: PublicKey<SumEd25519_12>,
    #[serde(deserialize_with = "jormungandr_utils::serde::crypto::deserialize_public")]
    vrf_key: PublicKey<Curve25519_2HashDH>,
}

#[derive(Clone, Deserialize)]
pub struct OwnerKey(
    #[serde(deserialize_with = "jormungandr_utils::serde::crypto::deserialize_public")]
    PublicKey<Ed25519>,
);

#[derive(Clone, Deserialize)]
pub struct StakePoolInfo {
    serial: u128,
    owners: Vec<OwnerKey>,
    initial_key: GenesisPraosPublic,
}

/// Node Secret(s)
#[derive(Clone, Deserialize)]
pub struct NodeSecret {
    pub bft: Option<Bft>,
    pub genesis: Option<GenesisPraos>,
}

/// Node Secret's Public parts
#[derive(Clone)]
pub struct NodePublic {
    pub block_publickey: PublicKey<Ed25519>,
}

custom_error! {pub NodeSecretFromFileError
    Io { source: std::io::Error } = "Cannot read node's secrets: {source}",
    Format { source: serde_yaml::Error } = "Invalid Node secret file: {source}",
}

impl NodeSecret {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<NodeSecret, NodeSecretFromFileError> {
        let file = std::fs::File::open(path)?;
        Ok(serde_yaml::from_reader(file)?)
    }

    pub fn bft(&self) -> Option<BftLeader> {
        self.bft.clone().map(|bft| BftLeader {
            sig_key: bft.signing_key,
        })
    }

    pub fn genesis(&self) -> Option<GenesisLeader> {
        self.genesis.clone().map(|genesis| GenesisLeader {
            node_id: genesis.node_id.into(),
            sig_key: genesis.sig_key,
            vrf_key: genesis.vrf_key,
        })
    }
}
