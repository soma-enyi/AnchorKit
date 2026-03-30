#![cfg(test)]

mod capability_detection_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

    use crate::contract::{
        AnchorKitContract, AnchorKitContractClient, ServiceType,
        SERVICE_DEPOSITS, SERVICE_WITHDRAWALS, SERVICE_QUOTES, SERVICE_KYC,
    };
    use crate::errors::ErrorCode;

    fn make_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    fn setup(env: &Env) -> (AnchorKitContractClient, Address) {
        let contract_id = env.register_contract(None, AnchorKitContract);
        let client = AnchorKitContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        client.initialize(&admin);
        (client, admin)
    }

    fn services(env: &Env, vals: &[u32]) -> Vec<u32> {
        let mut v = Vec::new(env);
        for &s in vals {
            v.push_back(s);
        }
        v
    }

    fn register(client: &AnchorKitContractClient, anchor: &Address) {
        let session_id = client.create_session(anchor);
        client.register_attestor_with_session(&session_id, anchor);
    }

    // -----------------------------------------------------------------------
    // ServiceType enum
    // -----------------------------------------------------------------------

    #[test]
    fn test_service_type_values() {
        assert_eq!(ServiceType::Deposits.as_u32(), SERVICE_DEPOSITS);
        assert_eq!(ServiceType::Withdrawals.as_u32(), SERVICE_WITHDRAWALS);
        assert_eq!(ServiceType::Quotes.as_u32(), SERVICE_QUOTES);
        assert_eq!(ServiceType::KYC.as_u32(), SERVICE_KYC);
        assert_eq!(SERVICE_DEPOSITS, 1u32);
        assert_eq!(SERVICE_WITHDRAWALS, 2u32);
        assert_eq!(SERVICE_QUOTES, 3u32);
        assert_eq!(SERVICE_KYC, 4u32);
    }

    #[test]
    fn test_detect_deposit_only_anchor() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        client.configure_services(&anchor, &services(&env, &[SERVICE_DEPOSITS]));
        let record = client.get_supported_services(&anchor);
        assert_eq!(record.services.len(), 1);
        assert!(record.services.contains(&SERVICE_DEPOSITS));
        assert!(client.supports_service(&anchor, &SERVICE_DEPOSITS));
        assert!(!client.supports_service(&anchor, &SERVICE_WITHDRAWALS));
        assert!(!client.supports_service(&anchor, &SERVICE_QUOTES));
        assert!(!client.supports_service(&anchor, &SERVICE_KYC));
    }

    #[test]
    fn test_detect_withdrawal_only_anchor() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        client.configure_services(&anchor, &services(&env, &[SERVICE_WITHDRAWALS]));
        assert!(!client.supports_service(&anchor, &SERVICE_DEPOSITS));
        assert!(client.supports_service(&anchor, &SERVICE_WITHDRAWALS));
        assert!(!client.supports_service(&anchor, &SERVICE_QUOTES));
        assert!(!client.supports_service(&anchor, &SERVICE_KYC));
    }

    #[test]
    fn test_detect_quote_provider_anchor() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        client.configure_services(&anchor, &services(&env, &[SERVICE_QUOTES]));
        assert!(!client.supports_service(&anchor, &SERVICE_DEPOSITS));
        assert!(!client.supports_service(&anchor, &SERVICE_WITHDRAWALS));
        assert!(client.supports_service(&anchor, &SERVICE_QUOTES));
        assert!(!client.supports_service(&anchor, &SERVICE_KYC));
    }

    #[test]
    fn test_detect_full_service_anchor() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        client.configure_services(
            &anchor,
            &services(&env, &[SERVICE_DEPOSITS, SERVICE_WITHDRAWALS, SERVICE_QUOTES, SERVICE_KYC]),
        );
        assert!(client.supports_service(&anchor, &SERVICE_DEPOSITS));
        assert!(client.supports_service(&anchor, &SERVICE_WITHDRAWALS));
        assert!(client.supports_service(&anchor, &SERVICE_QUOTES));
        assert!(client.supports_service(&anchor, &SERVICE_KYC));
    }

    #[test]
    fn test_update_anchor_capabilities() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        client.configure_services(&anchor, &services(&env, &[SERVICE_DEPOSITS]));
        assert!(client.supports_service(&anchor, &SERVICE_DEPOSITS));
        assert!(!client.supports_service(&anchor, &SERVICE_WITHDRAWALS));
        client.configure_services(&anchor, &services(&env, &[SERVICE_DEPOSITS, SERVICE_WITHDRAWALS]));
        assert!(client.supports_service(&anchor, &SERVICE_DEPOSITS));
        assert!(client.supports_service(&anchor, &SERVICE_WITHDRAWALS));
    }

    #[test]
    fn test_reject_empty_services() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        let result = client.try_configure_services(&anchor, &services(&env, &[]));
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_duplicate_services() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        let result = client.try_configure_services(
            &anchor,
            &services(&env, &[SERVICE_DEPOSITS, SERVICE_DEPOSITS]),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_unregistered_anchor_services() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        let result = client.try_configure_services(&anchor, &services(&env, &[SERVICE_DEPOSITS]));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_services_for_non_configured_anchor() {
        let env = make_env();
        let (client, _) = setup(&env);
        let anchor = Address::generate(&env);
        register(&client, &anchor);
        let result = client.try_get_supported_services(&anchor);
        assert!(result.is_err());
    }
}
