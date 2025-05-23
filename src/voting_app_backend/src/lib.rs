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