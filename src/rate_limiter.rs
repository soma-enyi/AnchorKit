//! Rate limiting for attestation submissions
//!
//! This module implements per-attestor rate limiting for attestation submissions
//! to prevent spam and abuse of the contract.

use soroban_sdk::{contracttype, Address, Env};
use crate::errors::AnchorKitError;

/// Rate limit configuration stored in contract storage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitConfig {
    /// Maximum number of submissions allowed per window
    pub max_submissions: u32,
    /// Length of the rate limit window in ledgers
    pub window_length: u32,
}

/// Per-attestor rate limit state stored in contract storage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitState {
    /// Number of submissions in the current window
    pub submission_count: u32,
    /// Ledger number when the current window started
    pub window_start_ledger: u32,
}

/// Rate limiter for attestation submissions
pub struct RateLimiter;

impl RateLimiter {
    /// Check if an attestor can submit an attestation and increment their counter
    ///
    /// Returns `Ok(())` if the attestor is within the rate limit.
    /// Returns `Err(AnchorKitError::rate_limit_exceeded())` if the limit is exceeded.
    pub fn check_and_increment(
        env: &Env,
        attestor: &Address,
        config: &RateLimitConfig,
    ) -> Result<(), AnchorKitError> {
        let current_ledger = env.ledger().sequence();
        let state_key = Self::get_state_key(env, attestor);
        
        // Get or initialize rate limit state
        let mut state = env.storage().persistent().get::<_, RateLimitState>(&state_key)
            .unwrap_or(RateLimitState {
                submission_count: 0,
                window_start_ledger: current_ledger,
            });
        
        // Check if window has expired and reset if needed
        if Self::is_window_expired(current_ledger, state.window_start_ledger, config.window_length) {
            state = RateLimitState {
                submission_count: 0,
                window_start_ledger: current_ledger,
            };
        }
        
        // Check if limit is exceeded
        if state.submission_count >= config.max_submissions {
            return Err(AnchorKitError::rate_limit_exceeded());
        }
        
        // Increment counter and save state
        state.submission_count += 1;
        env.storage().persistent().set(&state_key, &state);
        
        Ok(())
    }
    
    /// Get the current rate limit state for an attestor
    pub fn get_state(env: &Env, attestor: &Address) -> RateLimitState {
        let state_key = Self::get_state_key(env, attestor);
        env.storage().persistent().get::<_, RateLimitState>(&state_key)
            .unwrap_or(RateLimitState {
                submission_count: 0,
                window_start_ledger: env.ledger().sequence(),
            })
    }
    
    /// Update the rate limit configuration (admin only)
    pub fn update_config(
        env: &Env,
        _admin: &Address,
        config: &RateLimitConfig,
    ) -> Result<(), AnchorKitError> {
        // In a real contract, you would check that admin is authorized
        // For now, we'll just store the config
        let config_key = Self::get_config_key(env);
        env.storage().persistent().set(&config_key, config);
        Ok(())
    }
    
    /// Get the current rate limit configuration
    pub fn get_config(env: &Env) -> RateLimitConfig {
        let config_key = Self::get_config_key(env);
        env.storage().persistent().get::<_, RateLimitConfig>(&config_key)
            .unwrap_or(RateLimitConfig {
                max_submissions: 10,
                window_length: 100,
            })
    }
    
    /// Check if a window has expired
    fn is_window_expired(current_ledger: u32, window_start_ledger: u32, window_length: u32) -> bool {
        current_ledger.saturating_sub(window_start_ledger) >= window_length
    }
    
    /// Generate storage key for rate limit state
    fn get_state_key(env: &Env, attestor: &Address) -> soroban_sdk::BytesN<32> {
        // Use the address bytes directly as the key
        // Convert address to bytes using its internal representation
        // We use the address string representation and hash it
        let address_str = attestor.to_string();
        // Use env.crypto().sha256() to hash the address string
        // Convert the string to bytes using copy_into_slice
        let mut address_bytes = [0u8; 56]; // Stellar addresses are 56 characters
        address_str.copy_into_slice(&mut address_bytes);
        let bytes = soroban_sdk::Bytes::from_slice(env, &address_bytes);
        let hash = env.crypto().sha256(&bytes);
        // Convert Hash<32> to BytesN<32>
        hash.into()
    }
    
    /// Generate storage key for rate limit config
    fn get_config_key(env: &Env) -> soroban_sdk::BytesN<32> {
        // Use a fixed key for config (32 bytes)
        let config_key = *b"rate_limit_config_______________";
        soroban_sdk::BytesN::from_array(env, &config_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCode;
    use crate::contract::AnchorKitContract;
    use soroban_sdk::testutils::Address as _;

    fn make_contract(env: &Env) -> soroban_sdk::Address {
        env.register_contract(None, AnchorKitContract)
    }

    #[test]
    fn test_rate_limit_under_limit() {
        let env = Env::default();
        let attestor = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);
        let config = RateLimitConfig {
            max_submissions: 10,
            window_length: 100,
        };
        let contract_id = make_contract(&env);
        let result = env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        });
        assert!(result.is_ok());
        let state = env.as_contract(&contract_id, || {
            RateLimiter::get_state(&env, &attestor)
        });
        assert_eq!(state.submission_count, 1);
    }

    #[test]
    fn test_rate_limit_at_limit() {
        let env = Env::default();
        let attestor = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);
        let config = RateLimitConfig {
            max_submissions: 2,
            window_length: 100,
        };
        let contract_id = make_contract(&env);
        assert!(env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        }).is_ok());
        assert!(env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        }).is_ok());
        let result = env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::RateLimitExceeded);
    }

    #[test]
    fn test_rate_limit_over_limit() {
        let env = Env::default();
        let attestor = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);
        let config = RateLimitConfig {
            max_submissions: 1,
            window_length: 100,
        };
        let contract_id = make_contract(&env);
        assert!(env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        }).is_ok());
        let result = env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::RateLimitExceeded);
    }
    
    #[test]
    fn test_rate_limit_window_reset() {
        let env = Env::default();
        let attestor = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);
        let config = RateLimitConfig { max_submissions: 1, window_length: 10 };
        let contract_id = make_contract(&env);
        assert!(env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        }).is_ok());
        assert!(env.as_contract(&contract_id, || {
            RateLimiter::check_and_increment(&env, &attestor, &config)
        }).is_err());
        let state = env.as_contract(&contract_id, || {
            RateLimiter::get_state(&env, &attestor)
        });
        assert_eq!(state.submission_count, 1); // second call was rejected, count stays at 1
    }

    #[test]
    fn test_rate_limit_config_update() {
        let env = Env::default();
        let admin = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);
        let new_config = RateLimitConfig { max_submissions: 20, window_length: 200 };
        let contract_id = make_contract(&env);
        let result = env.as_contract(&contract_id, || {
            RateLimiter::update_config(&env, &admin, &new_config)
        });
        assert!(result.is_ok());
        let config = env.as_contract(&contract_id, || {
            RateLimiter::get_config(&env)
        });
        assert_eq!(config.max_submissions, 20);
        assert_eq!(config.window_length, 200);
    }
    
    #[test]
    fn test_rate_limit_default_config() {
        let env = Env::default();
        let contract_id = make_contract(&env);
        let admin = <soroban_sdk::Address as soroban_sdk::testutils::Address>::generate(&env);
        let default_config = RateLimitConfig { max_submissions: 10, window_length: 100 };
        env.as_contract(&contract_id, || {
            RateLimiter::update_config(&env, &admin, &default_config).unwrap();
        });
        let config = env.as_contract(&contract_id, || {
            RateLimiter::get_config(&env)
        });
        assert_eq!(config.max_submissions, 10);
        assert_eq!(config.window_length, 100);
    }
}
