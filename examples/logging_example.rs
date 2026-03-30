use anchorkit::{AnchorKitContract, AnchorKitContractClient};
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

fn main() {
    println!("🚀 AnchorKit Logging Example");
    println!("=============================\n");

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AnchorKitContract);
    let client = AnchorKitContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    println!("✅ Contract initialized");

    let anchor = Address::generate(&env);
    let dummy_token = String::from_str(&env, "dummy.jwt.token");
    let issuer = Address::generate(&env);
    client.register_attestor(&anchor, &dummy_token, &issuer);
    println!("✅ Anchor registered");

    let request_id = client.generate_request_id();
    println!("✅ Request ID generated: {:?}", request_id.id);

    println!("\n🎉 Logging example completed!");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_logging_example() {
        main();
    }
}
