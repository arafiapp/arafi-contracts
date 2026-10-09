#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

const ADMIN: Symbol = symbol_short!("ADMIN");

#[contracttype]
#[derive(Clone)]
pub struct Payment {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub token: Address,
    pub reference: soroban_sdk::String,
    pub timestamp: u64,
    pub settled: bool,
}

#[contracttype]
pub enum DataKey {
    Payment(u64),
    PaymentCount,
}

/// ArafiPayment — on-chain payment settlement contract.
///
/// Enables merchants to create and settle payments denominated in any SEP-41
/// compatible token on Stellar (e.g., USDC, EURC). Each payment is recorded
/// with a reference string that maps back to the Arafi off-chain order ID.
#[contract]
pub struct ArafiPaymentContract;

#[contractimpl]
impl ArafiPaymentContract {
    /// Initialise the contract. Must be called once after deployment.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialised");
        }
        env.storage().instance().set(&ADMIN, &admin);
    }

    /// Create a payment and pull funds into escrow.
    /// Caller must pre-approve the contract to spend `amount` of `token`.
    /// Returns the new payment ID.
    pub fn create_payment(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        amount: i128,
        reference: soroban_sdk::String,
    ) -> u64 {
        sender.require_auth();
        if amount <= 0 { panic!("amount must be positive"); }

        let count_key = DataKey::PaymentCount;
        let payment_id: u64 = env.storage().persistent().get(&count_key).unwrap_or(0u64) + 1;

        let payment = Payment {
            id: payment_id,
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount,
            token: token.clone(),
            reference,
            timestamp: env.ledger().timestamp(),
            settled: false,
        };

        let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        env.storage().persistent().set(&DataKey::Payment(payment_id), &payment);
        env.storage().persistent().set(&count_key, &payment_id);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("payment"), symbol_short!("created")),
            (payment_id, sender, recipient, amount),
        );

        payment_id
    }

    /// Settle a payment — admin only. Releases escrowed funds to recipient.
    pub fn settle(env: Env, payment_id: u64) {
        let admin: Address = env.storage().instance().get(&ADMIN).expect("not initialised");
        admin.require_auth();

        let key = DataKey::Payment(payment_id);
        let mut payment: Payment = env.storage().persistent().get(&key).expect("payment not found");
        if payment.settled { panic!("already settled"); }

        payment.settled = true;
        env.storage().persistent().set(&key, &payment);

        let token_client = soroban_sdk::token::TokenClient::new(&env, &payment.token);
        token_client.transfer(&env.current_contract_address(), &payment.recipient, &payment.amount);

        #[allow(deprecated)]
        env.events().publish(
            (symbol_short!("payment"), symbol_short!("settled")),
            (payment_id, payment.recipient, payment.amount),
        );
    }

    pub fn get_payment(env: Env, payment_id: u64) -> Payment {
        env.storage().persistent().get(&DataKey::Payment(payment_id)).expect("payment not found")
    }

    pub fn payment_count(env: Env) -> u64 {
        env.storage().persistent().get(&DataKey::PaymentCount).unwrap_or(0u64)
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN).expect("not initialised")
    }
}

mod test;
