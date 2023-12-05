use ciborium::ser;
use ic_stable_structures::writer::Writer;

use shared_utils::{common::utils::stable_memory_serializer_deserializer, canister_specific::individual_user_template::memory};
use crate::CANISTER_DATA;

pub const BUFFER_SIZE_BYTES: usize = 2 * 1024 * 1024; // 2 MiB

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    let mut state_bytes = vec![];
    CANISTER_DATA.with(|canister_data_ref_cell| {
        ser::into_writer(&*canister_data_ref_cell.borrow(), &mut state_bytes)
    })
    .expect("failed to encode state");

    let len = state_bytes.len() as u32;
    let mut memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();
}
