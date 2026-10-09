#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

const ADMIN: Symbol = symbol_short!("ADMIN");

#[contracttype]
#[derive(Clone)]
pub struct Subscription {
    pub id: u64,
    pub subscriber: Address,
    pub merchant: Address,
    pub token: Address,
    /// Recurring charge amount per cycle (token's smallest unit)
    pub amount: i128,
    /// Billing interval in seconds (e.g. 2_592_000 = 30 days)
    pub interval: u64,
    /// Ledger timestamp of last successful charge
    pub last_charged: u64,
    pub created_at: u64,
    pub active: bool,
}

#[contracttype]
pub enum DataKey {
    Subscription(u64),
    SubCount,
}

/// ArafiSubscription — on-chain recurring billing contract.
///
/// Merchants register subscription plans. Arafi's backend (admin) triggers
/// `charge` on each billing cycle. Subscribers can cancel at any time.
#[contract]
pub struct ArafiSubscriptionContract;

#[contractimpl]
impl ArafiSubscriptionContract {
    /// Initialise the contract. Must be called once after deployment.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN) { panic!("already initialised"); }
        env.storage().instance().set(&ADMIN, &admin);
    }

    /// Register a new subscription and charge the first cycle immediately.
    /// Subscriber must pre-approve the contract for a large allowance.
    /// Returns the subscription ID.
    pub fn subscribe(
        env: Env,
        subscriber: Address,
        merchant: Address,
        token: Address,
        amount: i128,
        interval: u64,
    ) -> u64 {
        subscriber.require_auth();
        if amount <= 0 { panic!("amount must be positive"); }
        if interval == 0 { panic!("interval must be non-zero"); }

        let count_key = DataKey::SubCount;
        let sub_id: u64 = env.storage().persistent().get(&count_key).unwrap_or(0u64) + 1;

        let now = env.ledger().timestamp();
        let sub = Subscription {
            id: sub_id,
            subscriber: subscriber.clone(),
            merchant: merchant.clone(),
            token: token.clone(),
            amount,
            interval,
            last_charged: now,
            created_at: now,
            active: true,
        };

        let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
        token_client.transfer(&subscriber, &merchant, &amount);

        env.storage().persistent().set(&DataKey::Subscription(sub_id), &sub);
        env.storage().persistent().set(&count_key, &sub_id);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("sub"), symbol_short!("created")),
            (sub_id, subscriber, merchant, amount, interval),
        );

        sub_id
    }

    /// Charge a subscription — admin only.
    /// Reverts if the billing interval has not elapsed.
    pub fn charge(env: Env, sub_id: u64) {
        let admin: Address = env.storage().instance().get(&ADMIN).expect("not initialised");
        admin.require_auth();

        let key = DataKey::Subscription(sub_id);
        let mut sub: Subscription = env.storage().persistent().get(&key).expect("subscription not found");

        if !sub.active { panic!("subscription is cancelled"); }

        let now = env.ledger().timestamp();
        if now < sub.last_charged + sub.interval { panic!("billing interval not elapsed"); }

        let token_client = soroban_sdk::token::TokenClient::new(&env, &sub.token);
        token_client.transfer(&sub.subscriber, &sub.merchant, &sub.amount);

        sub.last_charged = now;
        env.storage().persistent().set(&key, &sub);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("sub"), symbol_short!("charged")),
            (sub_id, sub.subscriber, sub.amount),
        );
    }

    /// Cancel a subscription. Callable by subscriber or admin.
    pub fn cancel(env: Env, caller: Address, sub_id: u64) {
        caller.require_auth();

        let key = DataKey::Subscription(sub_id);
        let mut sub: Subscription = env.storage().persistent().get(&key).expect("subscription not found");

        let admin: Address = env.storage().instance().get(&ADMIN).expect("not initialised");
        if sub.subscriber != caller && admin != caller { panic!("not authorised to cancel"); }
        if !sub.active { panic!("already cancelled"); }

        sub.active = false;
        env.storage().persistent().set(&key, &sub);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("sub"), symbol_short!("cancelled")),
            (sub_id, caller),
        );
    }

    pub fn get_subscription(env: Env, sub_id: u64) -> Subscription {
        env.storage().persistent().get(&DataKey::Subscription(sub_id)).expect("subscription not found")
    }

    pub fn subscription_count(env: Env) -> u64 {
        env.storage().persistent().get(&DataKey::SubCount).unwrap_or(0u64)
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN).expect("not initialised")
    }
}

mod test;
