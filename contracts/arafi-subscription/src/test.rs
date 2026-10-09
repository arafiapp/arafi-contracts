#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

fn create_token<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(env, &sac.address()),
        StellarAssetClient::new(env, &sac.address()),
    )
}

#[test]
fn test_subscribe_and_charge() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ArafiSubscriptionContract, ());
    let client = ArafiSubscriptionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);

    let (token, token_admin) = create_token(&env, &admin);
    token_admin.mint(&subscriber, &10_000_0000000i128);

    // Approve contract for recurring pulls
    token.approve(&subscriber, &contract_id, &10_000_0000000i128, &10_000);

    client.init(&admin);

    let interval: u64 = 30 * 24 * 60 * 60; // 30 days
    let amount: i128 = 10_0000000; // 10 tokens

    let sub_id = client.subscribe(&subscriber, &merchant, &token.address, &amount, &interval);
    assert_eq!(sub_id, 1u64);

    // Initial charge happens on subscribe
    assert_eq!(token.balance(&merchant), amount);

    // Advance ledger time by 30 days
    env.ledger().with_mut(|l| {
        l.timestamp += interval;
    });

    client.charge(&sub_id);

    // Merchant should now have 2x amount
    assert_eq!(token.balance(&merchant), amount * 2);
}

#[test]
#[should_panic(expected = "billing interval not elapsed")]
fn test_charge_before_interval_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ArafiSubscriptionContract, ());
    let client = ArafiSubscriptionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);

    let (token, token_admin) = create_token(&env, &admin);
    token_admin.mint(&subscriber, &10_000_0000000i128);
    token.approve(&subscriber, &contract_id, &10_000_0000000i128, &10_000);

    client.init(&admin);
    let sub_id = client.subscribe(
        &subscriber,
        &merchant,
        &token.address,
        &10_0000000i128,
        &(30 * 24 * 60 * 60),
    );

    // Charge immediately without advancing time — should panic
    client.charge(&sub_id);
}

#[test]
#[should_panic(expected = "subscription is cancelled")]
fn test_charge_after_cancel_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ArafiSubscriptionContract, ());
    let client = ArafiSubscriptionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);

    let (token, token_admin) = create_token(&env, &admin);
    token_admin.mint(&subscriber, &10_000_0000000i128);
    token.approve(&subscriber, &contract_id, &10_000_0000000i128, &10_000);

    client.init(&admin);
    let interval: u64 = 30 * 24 * 60 * 60;
    let sub_id = client.subscribe(
        &subscriber,
        &merchant,
        &token.address,
        &10_0000000i128,
        &interval,
    );

    // Cancel the subscription
    client.cancel(&subscriber, &sub_id);

    // Advance time past interval
    env.ledger().with_mut(|l| {
        l.timestamp += interval;
    });

    // Charge should now panic
    client.charge(&sub_id);
}
