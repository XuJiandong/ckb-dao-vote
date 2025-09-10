#![cfg_attr(not(any(test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

#[cfg(not(any(test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(test)))]
// By default, the following heap configuration is used:
// * 16KB fixed heap
// * 1.2MB(rounded up to be 16-byte aligned) dynamic heap
// * Minimal memory block in dynamic heap is 64 bytes
// For more details, please refer to ckb-std's default_alloc macro
// and the buddy-alloc alloc implementation.
ckb_std::default_alloc!(16384, 1258306, 64);

mod entry;
mod error;
mod molecules;
mod smt_hasher;

pub fn program_entry() -> i8 {
    #[cfg(feature = "enable_log")]
    {
        drop(ckb_std::logger::init());
        log::info!("ckb-dao-vote, log enabled");
    }
    match entry::entry() {
        Ok(_) => 0,
        Err(e) => {
            #[cfg(feature = "enable_log")]
            log::error!("error: {:?}", e);
            e.error_code()
        }
    }
}
