use blake2::digest::{CustomizedInit, FixedOutput};
use blake2::{Blake2b256, Digest};
use sparse_merkle_tree::{H256, traits::Hasher};

const PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub struct Blake2bHasher(Blake2b256);

impl Default for Blake2bHasher {
    fn default() -> Self {
        let blake2b = Blake2b256::new_customized(PERSONALIZATION);
        Blake2bHasher(blake2b)
    }
}

impl Hasher for Blake2bHasher {
    fn write_h256(&mut self, h: &H256) {
        self.0.update(h.as_slice());
    }
    fn write_byte(&mut self, b: u8) {
        self.0.update([b]);
    }
    fn finish(self) -> H256 {
        let result: [u8; 32] = self.0.finalize_fixed().as_slice().try_into().unwrap();
        result.into()
    }
}
