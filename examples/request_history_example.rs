use anchorkit::{AnchorKitContract, AnchorKitContractClient};
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

fn main() {
    println!("🚀 AnchorKit Request History Example");
    println!("=====================================\n");

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AnchorKitContract);
    let client = AnchorKitContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let anchor = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);

    let dummy_token = String::from_str(&env, "dummy.jwt.token");
    let issuer = Address::generate(&env);
    client.register_attestor(&anchor, &dummy_token, &issuer);

    // Submit attestations with request IDs
    let request_id1 = client.generate_request_id();
    let payload1 = Bytes::from_slice(&env, &[1u8; 32]);
    let sig = Bytes::new(&env);
    let id1 = client.submit_with_request_id(
        &request_id1,
        &anchor,
        &user,
        &env.ledger().timestamp(),
        &payload1,
        &sig,
    );
    println!("   ✅ Attestation 1 recorded (ID: {})", id1);

    let request_id2 = client.generate_request_id();
    let payload2 = Bytes::from_slice(&env, &[2u8; 32]);
    let id2 = client.submit_with_request_id(
        &request_id2,
        &anchor,
        &user,
        &env.ledger().timestamp(),
        &payload2,
        &sig,
    );
    println!("   ✅ Attestation 2 recorded (ID: {})\n", id2);

    println!("✅ Request history example completed!");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_request_history_example() {
        main();
    }
}
