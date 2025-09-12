use crate::{
    molecules::{VoteMeta, VoteProof, WitnessArgs},
    smt_hasher::Blake2bHasher,
    Loader,
};
use ckb_testtool::{
    builtin::ALWAYS_SUCCESS,
    ckb_hash::blake2b_256,
    ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*},
    context::Context,
};
use serde_molecule::to_vec;
use sparse_merkle_tree::{default_store::DefaultStore, SparseMerkleTree, H256};

const SMT_VALUE: [u8; 32] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub type CkbSMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

fn blake160(bytes: &[u8]) -> [u8; 20] {
    blake2b_256(bytes)[0..20].try_into().unwrap()
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
        .unwrap();

    let always_success_script = context
        .build_script(&always_success_out_point, Bytes::new())
        .unwrap();

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
    let outputs_data = vec![Bytes::from(vec![1, 0, 0, 0])];

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

#[test]
fn test_consume_vote() {
    // no need to attach vote meta cell
    let mut context = Context::default();
    let script_bin: Bytes = Loader::default().load_binary("ckb-dao-vote");
    let out_point = context.deploy_cell(script_bin);

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    let type_script = context.build_script(&out_point, Bytes::new()).unwrap();

    let always_success_script = context
        .build_script(&always_success_out_point, Bytes::new())
        .unwrap();

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(always_success_script.clone())
            .type_(Some(type_script).pack())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(500u64.pack())
        .lock(always_success_script.clone())
        .build()];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(vec![Bytes::new()].pack())
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
    VerifySmtNotOn,
    NoLockFound,
    WrongVoteCandidate,
    WrongVoteCandidateExceedLimit,
    MultipleCandidates,
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
            .unwrap();
        let script_hash = voter_lock_script.calc_script_hash();
        let key: [u8; 32] = script_hash.as_slice().try_into().unwrap();
        smt_tree
            .update(key.into(), SMT_VALUE.clone().into())
            .unwrap();
    }
    let smt_root_hash = match config.test_scheme {
        TestScheme::VerifySmtNotOn => [0u8; 32],
        _ => smt_tree.root().clone().into(),
    };
    let vote_meta = VoteMeta {
        smt_root_hash: Some(smt_root_hash),
        candidates: (0..config.candidate_count).map(|i| vec![i as u8]).collect(),
        start_time: 0,
        end_time: 0,
        extra: None,
    };
    let vote_meta_bin = to_vec(&vote_meta, false).unwrap();

    let vote_meta_bin = match config.test_scheme {
        TestScheme::Molecule => vote_meta_bin.iter().map(|_| 0).collect(),
        _ => vote_meta_bin,
    };
    let vote_meta_out_point = context.deploy_cell(vote_meta_bin.into());

    let script_bin: Bytes = Loader::default().load_binary("ckb-dao-vote");
    let out_point = context.deploy_cell(script_bin);

    let args = match config.test_scheme {
        TestScheme::NoMetaCell => [0u8; 20],
        _ => blake160(vote_meta_out_point.as_slice()),
    };
    let args: Vec<u8> = match config.test_scheme {
        TestScheme::WrongArgs => vec![0u8; 10],
        _ => args.to_vec(),
    };
    let type_script = context.build_script(&out_point, Bytes::from(args)).unwrap();
    let always_success_script = context
        .build_script(&always_success_out_point, Bytes::new())
        .unwrap();

    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut outputs_data = vec![];
    let mut witnesses = vec![];
    for i in 0..config.voter_count {
        // make args different to represent different voters
        let voter_lock_script = context
            .build_script(&always_success_out_point, Bytes::from(vec![i as u8]))
            .unwrap();

        let input_type_script = match config.test_scheme {
            TestScheme::WrongTxType => Some(type_script.clone()).pack(),
            _ => None::<Script>.pack(),
        };

        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(1000u64.pack())
                .lock(voter_lock_script.clone())
                .type_(input_type_script)
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
        match config.test_scheme {
            TestScheme::WrongVoteCandidate => {
                outputs_data.push(Bytes::from(vec![0, 0, 0, 1]).pack());
            }
            TestScheme::WrongVoteCandidateExceedLimit => {
                let data: u32 = 1 << config.candidate_count;
                outputs_data.push(Bytes::copy_from_slice(&data.to_le_bytes()).pack());
            }
            TestScheme::MultipleCandidates => {
                // select all choices
                let data: u32 = (1 << config.candidate_count) - 1;
                outputs_data.push(Bytes::copy_from_slice(&data.to_le_bytes()).pack());
            }
            _ => {
                outputs_data.push(Bytes::from(vec![1, 0, 0, 0]).pack());
            }
        }

        // witness
        let key: [u8; 32] = voter_lock_script
            .calc_script_hash()
            .as_slice()
            .try_into()
            .unwrap();
        let proof = smt_tree.merkle_proof(vec![key.into()]).unwrap();
        let compiled_proof = proof.clone().compile(vec![key.into()]).unwrap();

        let key = match config.test_scheme {
            TestScheme::NoLockFound => [0u8; 32],
            _ => key,
        };
        let smt_proof = match config.test_scheme {
            TestScheme::VerifySmtFail => vec![0u8; 1],
            _ => compiled_proof.0,
        };
        let vote_proof = VoteProof {
            lock_script_hash: key,
            smt_proof,
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

    let result = context.verify_tx(&tx, 10_000_000);
    match config.test_scheme {
        TestScheme::Normal | TestScheme::MultipleCandidates => {
            assert!(result.is_ok());
            println!("consume cycles: {}", result.unwrap());
        }
        _ => {
            assert!(result.is_err());
        }
    }
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

#[test]
fn test_wrong_choice() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::WrongVoteCandidate,
    });
}

#[test]
fn test_wrong_tx_type() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::WrongTxType,
    });
}

#[test]
fn test_no_meta_cell() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::NoMetaCell,
    });
}

#[test]
fn test_wrong_args() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::WrongArgs,
    });
}

#[test]
fn test_no_lock_found() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::NoLockFound,
    });
}

#[test]
fn test_verify_smt_fail() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::VerifySmtFail,
    });
}

#[test]
fn test_verify_smt_fail_not_on() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::VerifySmtNotOn,
    });
}

#[test]
fn test_molecule_failed() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::Molecule,
    });
}

#[test]
fn test_multiple_candidates() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::MultipleCandidates,
    });
}

#[test]
fn test_wrong_vote_candidate_exceed_limit() {
    entry(&Config {
        voter_count: 1,
        candidate_count: 5,
        test_scheme: TestScheme::WrongVoteCandidateExceedLimit,
    });
}
