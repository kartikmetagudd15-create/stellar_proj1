#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to store identity information
#[contracttype]
#[derive(Clone)]
pub struct Identity {
    pub user_address: Address,
    pub full_name: String,
    pub id_number: String,
    pub is_verified: bool,
    pub registration_time: u64,
    pub verification_time: u64,
}

// Mapping user address to their identity
#[contracttype]
pub enum IdentityBook {
    Identity(Address)
}

// Symbol for tracking total registered identities
const TOTAL_IDENTITIES: Symbol = symbol_short!("TOTAL_ID");

#[contract]
pub struct DigitalIdentityContract;

#[contractimpl]
impl DigitalIdentityContract {
    
    // Function 1: Register a new identity (called by user)
    pub fn register_identity(
        env: Env, 
        user: Address, 
        full_name: String, 
        id_number: String
    ) -> bool {
        // Verify the user is calling for themselves
        user.require_auth();
        
        // Check if identity already exists
        let key = IdentityBook::Identity(user.clone());
        let existing_identity: Option<Identity> = env.storage().instance().get(&key);
        
        if existing_identity.is_some() {
            log!(&env, "Identity already registered for this address");
            panic!("Identity already exists!");
        }
        
        // Get current timestamp
        let current_time = env.ledger().timestamp();
        
        // Create new identity record
        let new_identity = Identity {
            user_address: user.clone(),
            full_name,
            id_number,
            is_verified: false,
            registration_time: current_time,
            verification_time: 0,
        };
        
        // Store the identity
        env.storage().instance().set(&key, &new_identity);
        
        // Update total count
        let mut total: u64 = env.storage().instance().get(&TOTAL_IDENTITIES).unwrap_or(0);
        total += 1;
        env.storage().instance().set(&TOTAL_IDENTITIES, &total);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Identity registered successfully for address");
        true
    }
    
    // Function 2: Verify an identity (called by admin/verifier)
    pub fn verify_identity(env: Env, admin: Address, user_address: Address) -> bool {
        // Verify admin authorization
        admin.require_auth();
        
        // Get the identity record
        let key = IdentityBook::Identity(user_address.clone());
        let mut identity: Identity = env.storage().instance().get(&key)
            .expect("Identity not found!");
        
        // Check if already verified
        if identity.is_verified {
            log!(&env, "Identity already verified");
            return false;
        }
        
        // Update verification status
        identity.is_verified = true;
        identity.verification_time = env.ledger().timestamp();
        
        // Store updated identity
        env.storage().instance().set(&key, &identity);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Identity verified successfully");
        true
    }
    
    // Function 3: View identity details
    pub fn view_identity(env: Env, user_address: Address) -> Identity {
        let key = IdentityBook::Identity(user_address.clone());
        
        env.storage().instance().get(&key).unwrap_or(Identity {
            user_address: user_address.clone(),
            full_name: String::from_str(&env, "Not_Found"),
            id_number: String::from_str(&env, "Not_Found"),
            is_verified: false,
            registration_time: 0,
            verification_time: 0,
        })
    }
    
    // Function 4: Get total registered identities
    pub fn get_total_identities(env: Env) -> u64 {
        env.storage().instance().get(&TOTAL_IDENTITIES).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    
    #[test]
    fn test_register_and_verify() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DigitalIdentityContract);
        let client = DigitalIdentityContractClient::new(&env, &contract_id);
        
        let user = Address::generate(&env);
        let admin = Address::generate(&env);
        
        // Mock authorization
        env.mock_all_auths();
        
        // Register identity
        let result = client.register_identity(
            &user,
            &String::from_str(&env, "John Doe"),
            &String::from_str(&env, "ID123456")
        );
        assert_eq!(result, true);
        
        // Verify identity
        let verify_result = client.verify_identity(&admin, &user);
        assert_eq!(verify_result, true);
        
        // Check total
        let total = client.get_total_identities();
        assert_eq!(total, 1);
    }
}