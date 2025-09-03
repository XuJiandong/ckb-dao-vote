use crate::molecules::{VoteMeta, VoteProof, WitnessArgs};
use crate::smt_hasher::Blake2bHasher;
use crate::Loader;
use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::ckb_hash::new_blake2b;
use ckb_testtool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use ckb_testtool::context::Context;
use serde_molecule::to_vec;
use sparse_merkle_tree::default_store::DefaultStore;
use sparse_merkle_tree::{SparseMerkleTree, H256};

const SMT_VALUE: [u8; 32] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub type CkbSMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

fn blake160(bytes: &[u8]) -> [u8; 20] {
    let mut blake2b = new_blake2b();
    blake2b.update(bytes);
    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);
    ret[0..20].try_into().unwrap()
}

#[test]
fn test_open_vote() {
    // open vote doesn't require smt root hash
    let vote_meta = VoteMeta {
        smt_root_hash: None,
        candidates: vec![vec![0], vec![1]],
        start_time: 0,
        end_time: 0,
        extra: None,
    };
    let vote_meta_bin = to_vec(&vote_meta, false).expect("serialize vote meta");

    let mut context = Context::default();
    let vote_meta_out_point = context.deploy_cell(vote_meta_bin.into());

    let script_bin: Bytes = Loader::default().load_binary("ckb-dao-vote");
    let out_point = context.deploy_cell(script_bin);

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    let args = blake160(vote_meta_out_point.as_slice());
    let type_script = context
        .build_script(&out_point, Bytes::from(args.to_vec()))
        .expect("script");

    let always_success_script = context
        .build_script(&always_success_out_point, Bytes::new())
        .expect("always success script");

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(always_success_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(500u64.pack())
        .lock(always_success_script.clone())
        .type_(Some(type_script).pack())
        .build()];

    // this is the voter choice
    let outputs_data = vec![Bytes::from(vec![0])];

    let vote_proof = VoteProof {
        lock_script_hash: always_success_script
            .calc_script_hash()
            .as_slice()
            .try_into()
            .unwrap(),
        smt_proof: Vec::new(),
    };

    let witness_args = WitnessArgs {
        lock: None,
        input_type: None,
        output_type: Some(to_vec(&vote_proof, false).unwrap()),
    };
    let witness_args = Bytes::from(to_vec(&witness_args, false).unwrap());

    let vote_meta_cell_dep = CellDep::new_builder()
        .out_point(vote_meta_out_point)
        .dep_type(0u8.into())
        .build();
    let tx = TransactionBuilder::default()
        .cell_dep(vote_meta_cell_dep)
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness_args.pack())
        .build();
    let tx = context.complete_tx(tx);

    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

pub(crate) enum TestScheme {
    Normal,
    Molecule,
    WrongTxType,
    WrongArgs,
    NoMetaCell,
    VerifySmtFail,
    NoLockFound,
    WrongVoteCandidate,
}

pub(crate) struct Config {
    voter_count: usize,
    candidate_count: usize,
    test_scheme: TestScheme,
}

pub(crate) fn entry(config: &Config) {
    let mut context = Context::default();

    // collect all voter's lock script hashes into SMT
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let mut smt_tree = CkbSMT::default();
    for i in 0..config.voter_count {
        // make args different to represent different voters
        let voter_lock_script = context
            .build_script(&always_success_out_point, Bytes::from(vec![i as u8]))
            .expect("always success script");
        let script_hash = voter_lock_script.calc_script_hash();
        let key: [u8; 32] = script_hash.as_slice().try_into().unwrap();
        smt_tree
            .update(key.into(), SMT_VALUE.clone().into())
            .unwrap();
    }

    let vote_meta = VoteMeta {
        smt_root_hash: Some(smt_tree.root().clone().into()),
        candidates: (0..config.candidate_count).map(|i| vec![i as u8]).collect(),
        start_time: 0,
        end_time: 0,
        extra: None,
    };
    let vote_meta_bin = to_vec(&vote_meta, false).expect("serialize vote meta");
    let vote_meta_out_point = context.deploy_cell(vote_meta_bin.into());

    let script_bin: Bytes = Loader::default().load_binary("ckb-dao-vote");
    let out_point = context.deploy_cell(script_bin);

    let args = blake160(vote_meta_out_point.as_slice());
    let type_script = context
        .build_script(&out_point, Bytes::from(args.to_vec()))
        .expect("script");
    let always_success_script = context
        .build_script(&always_success_out_point, Bytes::new())
        .expect("always success script");

    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut outputs_data = vec![];
    let mut witnesses = vec![];
    for i in 0..config.voter_count {
        // make args different to represent different voters
        let voter_lock_script = context
            .build_script(&always_success_out_point, Bytes::from(vec![i as u8]))
            .expect("always success script");

        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(1000u64.pack())
                .lock(voter_lock_script.clone())
                .build(),
            Bytes::new(),
        );
        inputs.push(
            CellInput::new_builder()
                .previous_output(input_out_point)
                .build(),
        );
        outputs.push(
            CellOutput::new_builder()
                .capacity(500u64.pack())
                .lock(always_success_script.clone())
                .type_(Some(type_script.clone()).pack())
                .build(),
        );
        // choice
        outputs_data.push(Bytes::from(vec![0]).pack());

        // witness
        let key: [u8; 32] = voter_lock_script
            .calc_script_hash()
            .as_slice()
            .try_into()
            .unwrap();
        let proof = smt_tree.merkle_proof(vec![key.into()]).expect("proof");
        let compiled_proof = proof
            .clone()
            .compile(vec![key.into()])
            .expect("compile proof");
        let vote_proof = VoteProof {
            lock_script_hash: key,
            smt_proof: compiled_proof.0,
        };

        let witness_args = WitnessArgs {
            lock: None,
            input_type: None,
            output_type: Some(to_vec(&vote_proof, false).unwrap()),
        };
        let witness_args = Bytes::from(to_vec(&witness_args, false).unwrap());
        witnesses.push(witness_args.pack());
    }
    let vote_meta_cell_dep = CellDep::new_builder()
        .out_point(vote_meta_out_point)
        .dep_type(0u8.into())
        .build();
    let tx = TransactionBuilder::default()
        .cell_dep(vote_meta_cell_dep)
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data)
        .witnesses(witnesses)
        .build();
    let tx = context.complete_tx(tx);

    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_one_vote() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 1,
        test_scheme: TestScheme::Normal,
    });
}

#[test]
fn test_multiple_vote() {
    entry(&Config {
        voter_count: 3,
        candidate_count: 5,
        test_scheme: TestScheme::Normal,
    });
}
