use anchorkit::{AnchorKitContract, AnchorKitContractClient};
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, String};

fn main() {
    println!("🚀 AnchorKit CLI Example - Deposit/Withdraw Workflow");
    println!("==================================================\n");

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AnchorKitContract);
    let client = AnchorKitContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let anchor = Address::generate(&env);
    let user = Address::generate(&env);

    // Step 1: Initialize
    println!("1️⃣  Initializing contract...");
    client.initialize(&admin);
    println!("   ✅ Contract initialized\n");

    // Step 2: Register Anchor (with dummy SEP-10 token — mock_all_auths bypasses verification)
    println!("2️⃣  Registering anchor...");
    let dummy_token = String::from_str(&env, "dummy.jwt.token");
    let issuer = Address::generate(&env);
    client.register_attestor(&anchor, &dummy_token, &issuer);
    println!("   ✅ Anchor registered\n");

    // Step 3: Configure Services
    println!("3️⃣  Configuring anchor services...");
    let mut services = soroban_sdk::Vec::new(&env);
    services.push_back(1u32); // Deposits
    services.push_back(2u32); // Withdrawals
    client.configure_services(&anchor, &services);
    println!("   ✅ Services configured\n");

    // Step 4: Submit Deposit Attestation
    println!("4️⃣  Submitting deposit attestation...");
    let payload_hash = Bytes::from_slice(&env, &[1u8; 32]);
    let signature = Bytes::new(&env);
    let attestation_id = client.submit_attestation(
        &anchor,
        &user,
        &env.ledger().timestamp(),
        &payload_hash,
        &signature,
    );
    println!("   ✅ Deposit attestation recorded (ID: {})\n", attestation_id);

    // Step 5: Submit Withdraw Attestation
    println!("5️⃣  Submitting withdraw attestation...");
    let payload_hash2 = Bytes::from_slice(&env, &[2u8; 32]);
    let attestation_id2 = client.submit_attestation(
        &anchor,
        &user,
        &env.ledger().timestamp(),
        &payload_hash2,
        &signature,
    );
    println!("   ✅ Withdraw attestation recorded (ID: {})\n", attestation_id2);

    println!("✅ Workflow completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cli_example() {
        main();
    }
}
