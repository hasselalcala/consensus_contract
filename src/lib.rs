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
    participants: LookupMap<AccountId, ParticipantType>, // lista de participantes registrados en el protocolo
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            request: Vector::new(b"v"),
            participants: LookupMap::new(b"p"),
        }
    }

    pub fn register_participants(
        &mut self,
        participant_id: AccountId,
        type_participant: ParticipantType,
    ) {
        //TODO, verificar primero si no esta registrado ya
        //No deberia haber una funcion de registro ya que se supone ya estan registrados al protocolo
        self.participants.insert(participant_id, type_participant);
    }

    pub fn get_register_participants(
        &self,
        participant_id: &AccountId,
    ) -> Option<&ParticipantType> {
        self.participants.get(participant_id)
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

    //Use the request_id to know which request gonna vote
    pub fn commit_by_miner(&mut self, request_id: u64, answer: String) {
        //TODO: Función get_request usando request_id
        let complete_request = match self.find_request_by_id(request_id) {
            Some(request) => request,
            None => panic!("Request not found"),
        };

        // Verificar que estás a tiempo para comprometer
        require!(
            env::epoch_height() < complete_request.commit_miner_deadline,
            "No time to commit"
        );

        // Verificar si eres un minero e insertar la propuesta
        let allow_proposal = match self.get_register_participants(&env::predecessor_account_id()) {
            Some(type_participant) => {
                println!("Participant is {:?}", type_participant);
                *type_participant == ParticipantType::Miner
            }
            None => panic!("Not register"),
        };

        if allow_proposal {
            let proposal = MinerProposal {
                proposal_hash: env::keccak256(answer.as_bytes()),
                is_revealed: false,
            };

            // Insertar la propuesta en la solicitud encontrada
            complete_request
                .miners_proposals
                .insert(env::predecessor_account_id(), proposal);
        }
    }

    fn find_request_by_id(&mut self, request_id: u64) -> Option<&mut Request> {
        for request in &mut self.request {
            if request.request_id == request_id {
                return Some(request);
            }
        }
        None
    }

    //TODO: Answer in this method is a vector with the top ten
    pub fn commit_by_validator(&mut self, request_id: u64, answer: String) {
        // Buscar la solicitud por ID
        let complete_request = match self.find_request_by_id(request_id) {
            Some(request) => request,
            None => panic!("Request not found"),
        };

        // Verificar que estás a tiempo para comprometer
        require!(
            env::epoch_height() > complete_request.reveal_miner_deadline,
            "Miner commit time"
        );
        require!(
            env::epoch_height() < complete_request.commit_validator_deadline,
            "No time to commit"
        );

        // Verificar si eres un validador e insertar la propuesta
        match self.get_register_participants(&env::predecessor_account_id()) {
            Some(type_participant) => {
                println!("Participant is {:?}", type_participant);
                if *type_participant == ParticipantType::Validator {
                    let proposal = MinerProposal {
                        proposal_hash: env::keccak256(answer.as_bytes()),
                        is_revealed: false,
                    };

                    complete_request
                        .miners_proposals
                        .insert(env::predecessor_account_id(), proposal);
                } else {
                    panic!("You are not a validator");
                }
            }
            None => {
                panic!("Not registered");
            }
        }
    }

    pub fn reveal_by_miner(&mut self, request_id: u64, answer: String) {
        let complete_request = match self.find_request_by_id(request_id) {
            Some(request) => request,
            None => panic!("Request not found"),
        };

        require!(
            env::epoch_height() > complete_request.commit_miner_deadline,
            "commit time"
        );
        require!(
            env::epoch_height() < complete_request.reveal_miner_deadline,
            "No time to reveal"
        );

        //TODO.
        //verificar que la propuesta aun no está revelada.
        //Verificar que si tiene una propuesta en commit
        //vuelve a calcular el hash con la respuesta
        //Se compara el hash calculado con el hash almacenado
        //Si coincide, se cambia el revealed a true
        //Sino se dice que es una propuesta invalida porque no coinciden los hashes
    }
    pub fn reveal_by_validator() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_new(){

    // }

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
