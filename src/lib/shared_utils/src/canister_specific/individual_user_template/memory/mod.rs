use ic_stable_structures::{DefaultMemoryImpl, memory_manager::{MemoryId, VirtualMemory, MemoryManager, self}};
use std::{cell::RefCell};

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES: MemoryId = MemoryId::new(0);

// A memory for the StableBTreeMap we're using. A new memory should be created for
// every additional stable structure.
const STABLE_ROOM_DETAILS: MemoryId = MemoryId::new(1);

const STABLE_BET_DETAILS: MemoryId = MemoryId::new(2);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_upgrades_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(UPGRADES))
}

pub fn get_stable_bet_details() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(STABLE_BET_DETAILS))
}

pub fn get_stable_room_details() -> Memory {
    MEMORY_MANAGER.with(|mem| mem.borrow().get(STABLE_ROOM_DETAILS))
}