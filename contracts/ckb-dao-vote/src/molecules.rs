#[allow(clippy::all, unused_imports, dead_code)]
mod vote;

use crate::error::Error;
use alloc::boxed::Box;
use ckb_std::{ckb_constants::Source, error::SysError, syscalls};

pub use molecule::lazy_reader::{Cursor, Error as MoleculeError, Read};
pub use vote::*;

fn read_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    load_func: F,
    buf: &mut [u8],
    offset: usize,
    total_size: usize,
) -> Result<usize, MoleculeError> {
    if offset >= total_size {
        return Err(MoleculeError::OutOfBound(offset, total_size));
    }
    match load_func(buf, offset) {
        Ok(l) => Ok(l),
        Err(err) => match err {
            SysError::LengthNotEnough(_) => Ok(buf.len()),
            _ => Err(MoleculeError::OutOfBound(0, 0)),
        },
    }
}

fn read_size<F: Fn(&mut [u8]) -> Result<usize, SysError>>(
    load_func: F,
) -> Result<usize, MoleculeError> {
    let mut buf = [0u8; 4];
    match load_func(&mut buf) {
        Ok(l) => Ok(l),
        Err(e) => match e {
            SysError::LengthNotEnough(l) => Ok(l),
            _ => Err(MoleculeError::OutOfBound(0, 0)),
        },
    }
}

struct DataReader {
    total_size: usize,
    index: usize,
    source: Source,
}

impl DataReader {
    fn new(index: usize, source: Source) -> Self {
        let total_size = read_size(|buf| syscalls::load_cell_data(buf, 0, index, source)).unwrap();
        Self {
            total_size,
            source,
            index,
        }
    }
}

impl Read for DataReader {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, MoleculeError> {
        read_data(
            |buf, offset| syscalls::load_cell_data(buf, offset, self.index, self.source),
            buf,
            offset,
            self.total_size,
        )
    }
}

impl From<DataReader> for Cursor {
    fn from(data: DataReader) -> Self {
        Cursor::new(data.total_size, Box::new(data))
    }
}

pub struct WitnessArgsReader {
    total_size: usize,
    index: usize,
    source: Source,
}

impl WitnessArgsReader {
    pub fn new(index: usize, source: Source) -> Self {
        let total_size = read_size(|buf| syscalls::load_witness(buf, 0, index, source)).unwrap();
        Self {
            total_size,
            source,
            index,
        }
    }
}

impl Read for WitnessArgsReader {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, MoleculeError> {
        read_data(
            |buf, offset| syscalls::load_witness(buf, offset, self.index, self.source),
            buf,
            offset,
            self.total_size,
        )
    }
}

impl From<WitnessArgsReader> for Cursor {
    fn from(data: WitnessArgsReader) -> Self {
        Cursor::new(data.total_size, Box::new(data))
    }
}

fn load_witness_args(index: usize, source: Source) -> Result<vote::WitnessArgs, Error> {
    let reader = WitnessArgsReader::new(index, source);
    let cursor: Cursor = reader.into();
    let witness_args = WitnessArgs::from(cursor);
    witness_args.verify(false)?;
    Ok(witness_args)
}

pub fn load_vote_proof(index: usize) -> Result<vote::VoteProof, Error> {
    let witness_args = load_witness_args(index, Source::GroupOutput)?;
    let output_type = witness_args.output_type()?;
    let output_type = output_type.ok_or(Error::Molecule)?;
    let witness = VoteProof::from(output_type);
    witness.verify(false)?;
    Ok(witness)
}

pub fn load_vote_meta(index: usize) -> Result<VoteMeta, Error> {
    let reader = DataReader::new(index, Source::CellDep);
    let cursor: Cursor = reader.into();
    let data = VoteMeta::from(cursor);
    data.verify(false)?;

    Ok(data)
}
