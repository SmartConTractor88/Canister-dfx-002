#![allow(unused_imports)]
#![allow(dead_code)]

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager,
     VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap,
     storable::Bound, Storable};
use std::{borrow::Cow, cell::RefCell};
use serde::Deserialize;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUES: u32 = 5000;

#[derive(CandidType, Deserialize)]
enum Choice {
    Approve,
    Reject,
    Pass,
}

#[derive(CandidType)]
enum VotingError {
    AlreadyVoted,
    NotActive,
    NoSuchProposal,
    AccessRejected,
    UpdateError,
}

#[derive(CandidType, Deserialize, Clone)]
struct Proposal {
    description: String,
    approve: u32,
    reject: u32,
    pass: u32,
    active: bool,
    voted: Vec<candid::Principal>,
    owner: candid::Principal,
}

#[derive(CandidType, Deserialize)]
struct CreateProposal {
    description: String,
    active: bool,
}

impl Storable for Proposal {
    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUES,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static PROPOSAL_MAP: RefCell<StableBTreeMap<u64, Proposal, Memory>> =
        RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(
            |m| m.borrow().get(MemoryId::new(0))
        )));
}

#[ic_cdk::query]
fn get_proposal(key: u64) -> Option<Proposal> {
    PROPOSAL_MAP.with(|p| p.borrow().get(&key))
}

#[ic_cdk::query]
fn get_proposal_count() -> u64 {
    PROPOSAL_MAP.with(|p| p.borrow().len() as u64)
}

#[ic_cdk::update]
fn create_proposal(key: u64, proposal: CreateProposal) -> Option<Proposal> {
    let value = Proposal {
        description: proposal.description,
        approve: 0,
        reject: 0,
        pass: 0,
        active: proposal.active,
        voted: Vec::new(),
        owner: ic_cdk::caller(),
    };

    PROPOSAL_MAP.with(|p| {
        let mut map = p.borrow_mut();
        map.insert(key, value.clone())
    })
}

#[ic_cdk::update]
fn edit_proposal(key: u64, proposal: CreateProposal) -> Result<(), VotingError> {
    PROPOSAL_MAP.with(|p| {
        let old_proposal_opt = p.borrow().get(&key);
        let old_proposal: Proposal;

        match old_proposal_opt {
            Some(value) => old_proposal = value,
            None => return Err(VotingError::NoSuchProposal),
        }

        if ic_cdk::caller() != old_proposal.owner {
            return Err(VotingError::AccessRejected);
        }

        let value = Proposal {
            description: proposal.description,
            approve: old_proposal.approve,
            reject: old_proposal.reject,
            pass: old_proposal.pass,
            active: proposal.active,
            voted: old_proposal.voted,
            owner: old_proposal.owner,
        };

        let result = p.borrow_mut().insert(key, value);

        match result {
            Some(_) => Ok(()),
            None => Err(VotingError::UpdateError),
        }
    })
}

#[ic_cdk::update]
fn end_proposal(key: u64, _proposal: CreateProposal) -> Result<(), VotingError> {
    PROPOSAL_MAP.with(|p| {
        let proposal_opt = p.borrow().get(&key);
        let mut proposal: Proposal;

        match proposal_opt {
            Some(value) => proposal = value,
            None => return Err(VotingError::NoSuchProposal),
        }

        if ic_cdk::caller() != proposal.owner {
            return Err(VotingError::AccessRejected);
        }

        proposal.active = false;

        let result = p.borrow_mut().insert(key, proposal);

        match result {
            Some(_) => Ok(()),
            None => Err(VotingError::UpdateError),
        }
    })
}

#[ic_cdk::update]
fn vote(key: u64, choice: Choice) -> Result<(), VotingError> {
    PROPOSAL_MAP.with(|p| {
        let proposal_opt = p.borrow().get(&key);
        let mut proposal: Proposal;

        match proposal_opt {
            Some(value) => proposal = value,
            None => return Err(VotingError::NoSuchProposal),
        }

        let caller = ic_cdk::caller();

        if proposal.voted.contains(&caller) {
            return Err(VotingError::AlreadyVoted);
        } else if !proposal.active {
            return Err(VotingError::NotActive);
        }

        match choice {
            Choice::Approve => proposal.approve += 1,
            Choice::Reject => proposal.reject += 1,
            Choice::Pass => proposal.pass += 1,
        }

        proposal.voted.push(caller);

        let result = p.borrow_mut().insert(key, proposal);
        match result {
            Some(_) => Ok(()),
            None => Err(VotingError::UpdateError),
        }
    })
}