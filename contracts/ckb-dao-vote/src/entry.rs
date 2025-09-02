use crate::error::Error;
use crate::molecules::{load_vote_meta, load_vote_proof};
use crate::smt_hasher::Blake2bHasher;
use alloc::vec;
use alloc::vec::Vec;
use ckb_hash::new_blake2b;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::{Byte, Byte32};
use ckb_std::ckb_types::prelude::Entity;
use ckb_std::high_level::{
    QueryIter, load_cell_data, load_cell_lock_hash, load_cell_type, load_input_out_point,
    load_script,
};
use sparse_merkle_tree::CompiledMerkleProof;

const SMT_VALUE: [u8; 32] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn blake160(bytes: &[u8]) -> [u8; 20] {
    let mut blake2b = new_blake2b();
    blake2b.update(bytes);
    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);
    ret[0..20].try_into().unwrap()
}

fn count_matching_type_scripts(
    source: Source,
    target_code_hash: &Byte32,
    target_hash_type: Byte,
) -> usize {
    QueryIter::new(load_cell_type, source)
        .filter_map(|script_opt| script_opt)
        .filter(|script| {
            script.code_hash() == *target_code_hash && script.hash_type() == target_hash_type
        })
        .count()
}

pub(crate) fn entry() -> Result<(), Error> {
    let current_script = load_script()?;
    let current_code_hash = current_script.code_hash();
    let current_hash_type = current_script.hash_type();

    // step 1
    let input_count =
        count_matching_type_scripts(Source::Input, &current_code_hash, current_hash_type);
    let output_count =
        count_matching_type_scripts(Source::Output, &current_code_hash, current_hash_type);
    if input_count > 0 && output_count > 0 {
        return Err(Error::WrongTxType);
    }
    // step 2
    if input_count > 0 {
        // vote consumption
        return Ok(());
    }
    // vote creation
    let args: Vec<u8> = current_script.args().raw_data().into();
    if args.len() != 20 {
        return Err(Error::WrongArgs);
    }
    let args: [u8; 20] = args.try_into().unwrap();
    // TODO: cell deps can't be fetched from syscall directly
    let position = QueryIter::new(load_input_out_point, Source::CellDep)
        .position(|out| args == blake160(out.as_slice()));
    // step 3
    if position.is_none() {
        return Err(Error::NoMetaCell);
    }
    let position = position.unwrap();
    let vote_meta = load_vote_meta(position)?;
    let root_hash = vote_meta.smt_root_hash()?;

    if root_hash.is_none() {
        // All users can vote (open vote)
        // TODO: do we need to check step 6?
        return Ok(());
    }
    let root_hash: Vec<u8> = root_hash.unwrap().try_into()?;
    let root_hash: [u8; 32] = root_hash.try_into().map_err(|_| Error::WrongHashSize)?;

    let iter = QueryIter::new(load_cell_type, Source::GroupOutput);
    for (index, _) in iter.enumerate() {
        let vote_proof = load_vote_proof(index)?;
        let proof = vote_proof.smt_proof()?;
        let proof: Vec<u8> = proof.try_into()?;

        let hash: Vec<u8> = vote_proof.lock_script_hash()?.try_into()?;
        let hash: [u8; 32] = hash.try_into().map_err(|_| Error::WrongHashSize)?;
        let compiled_proof = CompiledMerkleProof(proof);
        // step 4
        compiled_proof
            .verify::<Blake2bHasher>(
                &root_hash.clone().into(),
                vec![(hash.clone().into(), SMT_VALUE.clone().into())],
            )
            .map_err(|_| Error::VerifySmtFail)?;

        // step 5
        if !QueryIter::new(load_cell_lock_hash, Source::Input).any(|lock| lock == hash) {
            return Err(Error::NoLockFound);
        }

        // step 6
        let cell_data = load_cell_data(index, Source::GroupOutput)?;
        if cell_data.len() != 1 {
            return Err(Error::WrongVoteCandidate);
        }
        let candidate = cell_data[0] as usize;
        if candidate >= vote_meta.candidates()?.len()? {
            return Err(Error::WrongVoteCandidate);
        }
    }

    Ok(())
}
