use ckb_std::error::SysError;
use core::fmt::Display;
use molecule::lazy_reader::Error as MoleculeError;

#[derive(Debug)]
pub enum Error {
    Syscall(SysError),
    Molecule,
    WrongTxType,
    WrongArgs,
    NoMetaCell,
    WrongHashSize,
    VerifySmtFail,
    NoLockFound,
    WrongVoteCandidate,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for Error {}

impl From<SysError> for Error {
    fn from(e: SysError) -> Self {
        Error::Syscall(e)
    }
}

impl From<MoleculeError> for Error {
    fn from(_: MoleculeError) -> Self {
        Error::Molecule
    }
}

impl Error {
    pub fn error_code(&self) -> i8 {
        match self {
            Error::Syscall(e) => match e {
                SysError::IndexOutOfBound => 21,
                SysError::ItemMissing => 22,
                SysError::LengthNotEnough(_) => 23,
                SysError::Encoding => 24,
                SysError::WaitFailure => 25,
                SysError::TypeIDError => 26,
                _ => 27,
            },
            Error::Molecule => 51,
            Error::WrongTxType => 52,
            Error::WrongArgs => 53,
            Error::NoMetaCell => 54,
            Error::WrongHashSize => 55,
            Error::VerifySmtFail => 56,
            Error::NoLockFound => 57,
            Error::WrongVoteCandidate => 58,
        }
    }
}
