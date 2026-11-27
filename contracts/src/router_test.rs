//! # Tests for StreamRouterContract cross-contract invocations

extern crate std;

use super::*;
use crate::router::{
    BatchStreamParam, SplitAllocation, StreamRouterContract, StreamRouterContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, Vec,
};

#[test]
fn test_router_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let stream_contract = env.register_contract(None, StellarStreamContract);
    let router_contract = env.register_contract(None, StreamRouterContract);
    let router_client = StreamRouterContractClient::new(&env, &router_contract);

    let admin = Address::generate(&env);
    router_client.initialize_router(&admin, &stream_contract);

    let stats = router_client.get_router_stats();
    assert_eq!(stats.total_batches_routed, 0);
    assert_eq!(stats.total_streams_routed, 0);
    assert_eq!(stats.total_volume_routed, 0);
}

#[test]
fn test_cross_contract_batch_creation() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Register contracts
    let stream_contract = env.register_contract(None, StellarStreamContract);
    let router_contract = env.register_contract(None, StreamRouterContract);
    let router_client = StreamRouterContractClient::new(&env, &router_contract);

    let admin = Address::generate(&env);
    router_client.initialize_router(&admin, &stream_contract);

    // 2. Setup token and sender balance
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let sender = Address::generate(&env);

    let total_deposit = 300_000_0000000i128; // 30,000 tokens
    token::StellarAssetClient::new(&env, &token).mint(&sender, &total_deposit);

    // 3. Prepare 3 batch stream parameters
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);

    let mut batch: Vec<BatchStreamParam> = Vec::new(&env);
    batch.push_back(BatchStreamParam {
        recipient: recipient1.clone(),
        token: token.clone(),
        total_amount: 100_000_0000000i128,
        duration_seconds: 10_000,
        cliff_seconds: 0,
    });
    batch.push_back(BatchStreamParam {
        recipient: recipient2.clone(),
        token: token.clone(),
        total_amount: 100_000_0000000i128,
        duration_seconds: 20_000,
        cliff_seconds: 1_000,
    });
    batch.push_back(BatchStreamParam {
        recipient: recipient3.clone(),
        token: token.clone(),
        total_amount: 100_000_0000000i128,
        duration_seconds: 30_000,
        cliff_seconds: 5_000,
    });

    // 4. Execute atomic batch creation via cross-contract calls
    let stream_ids = router_client.batch_create_streams(&sender, &stream_contract, &batch);
    assert_eq!(stream_ids.len(), 3);

    // 5. Verify stats updated
    let stats = router_client.get_router_stats();
    assert_eq!(stats.total_batches_routed, 1);
    assert_eq!(stats.total_streams_routed, 3);
    assert_eq!(stats.total_volume_routed, 300_000_0000000i128);

    // 6. Verify stream exists on StellarStreamContract
    let stream_client = StellarStreamContractClient::new(&env, &stream_contract);
    let s1 = stream_client.get_stream(&stream_ids.get(0).unwrap());
    assert_eq!(s1.recipient, recipient1);
    assert_eq!(s1.total_amount, 100_000_0000000i128);
}

#[test]
fn test_split_claim_end_to_end() {
    let env = Env::default();
    env.mock_all_auths();

    let stream_contract = env.register_contract(None, StellarStreamContract);
    let router_contract = env.register_contract(None, StreamRouterContract);
    let stream_client = StellarStreamContractClient::new(&env, &stream_contract);
    let router_client = StreamRouterContractClient::new(&env, &router_contract);

    let admin = Address::generate(&env);
    router_client.initialize_router(&admin, &stream_contract);

    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let sender = Address::generate(&env);
    let worker = Address::generate(&env);

    let total_deposit = 100_000_0000000i128; // 10,000 tokens
    token::StellarAssetClient::new(&env, &token).mint(&sender, &total_deposit);

    // Create stream vesting to router_contract over [0, 10_000]
    let stream_id = stream_client.create_stream(
        &sender,
        &router_contract,
        &token,
        &total_deposit,
        &0,
        &10_000,
        &0,
        &None,
    );

    // Advance ledger to 50% vested (5,000s -> 50,000_0000000 claimable)
    env.ledger().with_mut(|l| l.timestamp = 5_000);

    let dest_a = Address::generate(&env); // 80% personal wallet
    let dest_b = Address::generate(&env); // 20% tax withholding vault

    let mut allocs: Vec<SplitAllocation> = Vec::new(&env);
    allocs.push_back(SplitAllocation {
        destination: dest_a.clone(),
        basis_points: 8000,
    });
    allocs.push_back(SplitAllocation {
        destination: dest_b.clone(),
        basis_points: 2000,
    });

    let claimed = router_client.split_claim(&stream_contract, &stream_id, &worker, &token, &allocs);
    assert_eq!(claimed, 50_000_0000000i128);

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&dest_a), 40_000_0000000i128);
    assert_eq!(token_client.balance(&dest_b), 10_000_0000000i128);
}
