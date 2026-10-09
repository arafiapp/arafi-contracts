#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

// Helper: deploy a native Stellar Asset Contract (SAC) for testing
fn create_token<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(env, &sac.address()),
        StellarAssetClient::new(env, &sac.address()),
    )
}

#[test]
fn test_init_and_payment_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ArafiPaymentContract, ());
    let client = ArafiPaymentContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let customer = Address::generate(&env);

    // Deploy a test token and mint to customer
    let (token, token_admin) = create_token(&env, &admin);
    token_admin.mint(&customer, &1_000_0000000i128); // 1,000 units (7 dp)

    // Approve contract to pull funds
    token.approve(&customer, &contract_id, &100_0000000i128, &200);

    // Init contract
    client.init(&admin);
    assert_eq!(client.admin(), admin);

    // Create payment
    let ref_str = String::from_str(&env, "ORDER-001");
    let payment_id = client.create_payment(
        &customer,
        &merchant,
        &token.address,
        &50_0000000i128,
        &ref_str,
    );

    assert_eq!(payment_id, 1u64);
    assert_eq!(client.payment_count(), 1u64);

    let payment = client.get_payment(&payment_id);
    assert_eq!(payment.settled, false);
    assert_eq!(payment.amount, 50_0000000i128);

    // Settle payment
    client.settle(&payment_id);

    let settled = client.get_payment(&payment_id);
    assert_eq!(settled.settled, true);

    // Merchant should have received funds
    assert_eq!(token.balance(&merchant), 50_0000000i128);
}

#[test]
#[should_panic(expected = "already settled")]
fn test_double_settle_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ArafiPaymentContract, ());
    let client = ArafiPaymentContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let customer = Address::generate(&env);

    let (token, token_admin) = create_token(&env, &admin);
    token_admin.mint(&customer, &1_000_0000000i128);
    token.approve(&customer, &contract_id, &100_0000000i128, &200);

    client.init(&admin);
    let id = client.create_payment(
        &customer,
        &merchant,
        &token.address,
        &50_0000000i128,
        &String::from_str(&env, "ORDER-002"),
    );

    client.settle(&id);
    client.settle(&id); // should panic
}
