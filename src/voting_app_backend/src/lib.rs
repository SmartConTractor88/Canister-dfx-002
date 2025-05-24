#![allow(unused_imports)]
#![allow(dead_code)]

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager,
     VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap,
     Storable};
use std::{borrow::Cow, cell::RefCell};
use serde::Deserialize;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUES: u32 = 5000;

#[derive(CandidType, Deserialize)]
struct Proposal {
    description: String,
    approve: u32,
    reject: u32,
    pass: u32,
    active: bool, // if true, the proposal is active
    voted: Vec<candid::Principal>, // can vote only once
    owner: candid::Principal, 
}

#[derive(CandidType, Deserialize)]
struct CreateProposal {
    description: String,
    active: bool,
} // here are only the fields that the owner can set

impl Storable for Proposal {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Proposal {
    const MAX_SIZE: u32 = MAX_VALUES; // max size of the proposal description
    const IS_FIXED_SIZE: bool = false;
    fn to_bytes(&self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        Decode!(&bytes, Proposal).unwrap()
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = 
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()))

    static PROPOASL_MAP: RefCell<StableBTreeMap<u64, Proposal, Memory>> = 
    RefCell::new(StableBTreeMap::init(MemoryManager.with(
        |m| m.borrow().get(MemoryId::new(0)))))
}