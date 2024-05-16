/// Import `borsh` from `near_sdk` crate
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
/// Import `serde` from `near_sdk` crate
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::store::{LookupMap, Vector};
use near_sdk::{env, near_bindgen, require, AccountId, PanicOnDefault};

const COMMIT_MINER_DURATION_EPOCH: u64 = 5;
const REVEAL_MINER_DURATION_EPOCH: u64 = 1;
const COMMIT_VALIDATOR_DURATION_EPOCH: u64 = 5;
const REVEAL_VALIDATOR_DURATION_EPOCH: u64 = 1;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ParticipantType {
    Miner,
    Validator,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[derive(Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MinerProposal {
    proposal_hash: Vec<u8>,
    is_revealed: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorProposal {
    proposal_hash: Vec<u8>,
    is_revealed: bool,
    miner_addresses: Vec<AccountId>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
//#[serde(crate = "near_sdk::serde")]
pub struct Request {
    sender: AccountId,
    request_id: u64,
    start_time: u64,
    commit_miner_deadline: u64,
    reveal_miner_deadline: u64,
    commit_validator_deadline: u64,
    reveal_validator_deadline: u64,
    miners_proposals: LookupMap<AccountId, MinerProposal>,
    validators_proposals: LookupMap<AccountId, ValidatorProposal>,
}

/// Main contract structure serialized with Borsh
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
//#[serde(crate = "near_sdk::serde")]
pub struct Contract {
    request: Vector<Request>,
    miners: Vector<AccountId>,
    validators: Vector<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            request: Vector::new(b"r"),
            miners: Vector::new(b"m"),
            validators: Vector::new(b"v"),
        }
    }

    pub fn register_miner(&mut self, participant_id: AccountId) {
        
        let miner_to_register = self.get_register_validator(participant_id.clone());
        require!(miner_to_register.is_none());

        let miner_to_register= self.get_register_miner(participant_id.clone());
        require!(miner_to_register.is_none());
        
        self.miners.push(participant_id);
        
    }

    pub fn get_register_miner(&mut self, participant_id: AccountId) -> Option<&AccountId> {
        for miner in self.miners.iter(){
            if *miner == participant_id {
                return Some(miner);
            }
        }   
        None
    }

    pub fn register_validator(&mut self, participant_id: AccountId) {
        
        let validator_to_register = self.get_register_validator(participant_id.clone());
        require!(validator_to_register.is_none());

        let miner_to_register= self.get_register_miner(participant_id.clone());
        require!(miner_to_register.is_none());
        
        self.validators.push(participant_id);
    }

    pub fn get_register_validator(&mut self, participant_id: AccountId) -> Option<&AccountId> {
        for miner in self.miners.iter(){
            if *miner == participant_id {
                return Some(miner);
            }
        }   
        None
    }

    pub fn request_governance_decision(&mut self, request_id: u64) {
        let new_request = Request {
            sender: env::predecessor_account_id(),
            request_id: request_id,
            start_time: env::epoch_height(),
            commit_miner_deadline: env::epoch_height() + COMMIT_MINER_DURATION_EPOCH,
            reveal_miner_deadline: env::epoch_height()
                + COMMIT_MINER_DURATION_EPOCH
                + REVEAL_MINER_DURATION_EPOCH,
            commit_validator_deadline: env::epoch_height()
                + COMMIT_MINER_DURATION_EPOCH
                + REVEAL_MINER_DURATION_EPOCH
                + COMMIT_VALIDATOR_DURATION_EPOCH,
            reveal_validator_deadline: env::epoch_height()
                + COMMIT_MINER_DURATION_EPOCH
                + REVEAL_MINER_DURATION_EPOCH
                + COMMIT_VALIDATOR_DURATION_EPOCH
                + REVEAL_VALIDATOR_DURATION_EPOCH,
            miners_proposals: LookupMap::new(b"m"),
            validators_proposals: LookupMap::new(b"v"),
        };
        self.request.push(new_request);
    }

    fn get_request_by_id(&mut self, request_id: u64) -> Option<&mut Request> {
        for request in &mut self.request {
            if request.request_id == request_id {
                return Some(request);
            }
        }
        None
    }

    pub fn commit_by_miner(&mut self, miner: AccountId, request_id: u64, answer: String) {

        let miner_to_commit = self.get_register_miner(miner);
        require!(miner_to_commit.is_some());

        let request_exist = self.get_request_by_id(request_id);
        require!(request_exist.is_some());

        let complete_request = match self.get_request_by_id(request_id) {
            Some(request) => request,
            None => panic!("Request not found"),
        };

        require!(env::epoch_height() < complete_request.commit_miner_deadline,"No time to commit");

        let proposal = MinerProposal {
            proposal_hash: env::keccak256(answer.as_bytes()),
            is_revealed: false,    
        };

        complete_request.miners_proposals.insert(env::predecessor_account_id(), proposal);
        
    }

    //TODO: Answer in this method is a vector with the top ten
    pub fn commit_by_validator(&mut self, validator: AccountId, request_id: u64, answer: String) {
        
        let validator_to_commit = self.get_register_validator(validator);
        require!(validator_to_commit.is_some());

        let request_exist = self.get_request_by_id(request_id);
        require!(request_exist.is_some());

        let complete_request: &mut Request = match self.get_request_by_id(request_id) {
            Some(request) => request,
            None => panic!("Request not found"),
        };

        require!(env::epoch_height() > complete_request.reveal_miner_deadline, "Miner commit time");
        require!(env::epoch_height() < complete_request.commit_validator_deadline,"No time to commit");

        //TODO: answer is a vector with the list of miners to vote 
        let proposal = ValidatorProposal {
            proposal_hash: env::keccak256(answer.as_bytes()),
            is_revealed: false,
            miner_addresses: Vec::new(), 
        };
        complete_request.validators_proposals.insert(env::predecessor_account_id(), proposal);
                
    }

    pub fn reveal_by_miner(&mut self, miner: AccountId, request_id: u64, answer: String) {

        let request_exist = self.get_request_by_id(request_id);
        require!(request_exist.is_some());

        let miner_to_reveal = self.get_register_miner(miner.clone());
        require!(miner_to_reveal.is_some());

        let complete_request = match self.get_request_by_id(request_id) {
            Some(request) => request,
            None => panic!("Request not found"),
        };
        
        require!(env::epoch_height() > complete_request.commit_miner_deadline,"commit time");
        require!(env::epoch_height() < complete_request.reveal_miner_deadline,"No time to reveal");
        
        let save_proposal = match complete_request.miners_proposals.get_mut(&miner){
            Some(proposal) =>  proposal,
            None => panic!("proposal not found"),
         };

        require!(save_proposal.is_revealed == false, "Proposal already reveal");
        
        let answer_to_verify = env::keccak256(answer.as_bytes());
        require!(save_proposal.proposal_hash == answer_to_verify, "Answer don't match");

        save_proposal.is_revealed = true;
        
    }

    pub fn reveal_by_validator(&mut self, validator : AccountId, request_id: u64, answer: String) {

        let request_exist = self.get_request_by_id(request_id);
        require!(request_exist.is_some());

        let validator_to_reveal = self.get_register_miner(validator.clone());
        require!(validator_to_reveal.is_some());


        let complete_request = match self.get_request_by_id(request_id) {
            Some(request) => request,
            None => panic!("Request not found"),
        };
        
        require!(env::epoch_height() > complete_request.commit_validator_deadline,"commit time");
        require!(env::epoch_height() < complete_request.reveal_validator_deadline,"No time to reveal");

        let save_proposal = match complete_request.validators_proposals.get_mut(&validator){
            Some(proposal) =>  proposal,
            None => panic!("proposal not found"),
         };

        require!(save_proposal.is_revealed == false, "Proposal already reveal");
        
        let answer_to_verify = env::keccak256(answer.as_bytes());
        require!(save_proposal.proposal_hash == answer_to_verify, "Answer don't match");

        save_proposal.is_revealed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_miner() {

    }

    #[test]
    fn test_get_register_miner() {

    }

    #[test]
    fn test_register_validator() {

    }

    #[test]
    fn test_get_register_validator() {
        
    }

    #[test]
    fn test_request_governance_decision() {

    }


    #[test]
    fn test_commit_by_miner(){
        
    }

    #[test]
    fn test_commit_by_validator() {

    }

    #[test]
    fn test_get_request_by_id() {

    }

    #[test]
    fn test_reveal_by_miner() {

    }

    #[test]
    fn test_reveal_by_validator() {
        
    }

    #[test]
    fn test_register_participants() {
        let mut contract = Contract::new();
        let participant_1: AccountId = "alice.near".parse().unwrap();
        let participant_2: AccountId = "edson.near".parse().unwrap();

        contract.register_participants(participant_1.clone(), ParticipantType::Miner);
        contract.register_participants(participant_2.clone(), ParticipantType::Validator);

        match contract.get_register_participants(&participant_1) {
            Some(type_participant_1) => {
                println!("Participant is {:?}", type_participant_1);
            }
            None => {
                println!("not register");
            }
        }

        match contract.get_register_participants(&participant_2) {
            Some(type_participant_2) => {
                println!("Participant is: {:?}", type_participant_2);
            }
            None => {
                println!("not register");
            }
        }

        //TODO, que pasa si quiero volver a registrarme
    }

    // #[test]
    // fn test_get_register_participants(){

    // }
}
