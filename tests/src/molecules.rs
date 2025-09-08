use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WitnessArgs {
    pub lock: Option<Vec<u8>>,
    pub input_type: Option<Vec<u8>>,
    pub output_type: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteMeta {
    pub smt_root_hash: Option<[u8; 32]>,
    #[serde(with = "dynvec_serde")]
    pub candidates: Vec<Vec<u8>>,
    pub start_time: u64,
    pub end_time: u64,
    pub extra: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteProof {
    pub lock_script_hash: [u8; 32],
    pub smt_proof: Vec<u8>,
}
