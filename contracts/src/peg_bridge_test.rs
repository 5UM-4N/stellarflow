//! # Tests for 1:1 BNB <-> Stellar SAC Peg Bridge (`PegBridgeContract`)

extern crate std;

use super::*;
use crate::peg_bridge::{
    PegBridgeContract, PegBridgeContractClient, BNB18_TO_STELLAR7_DIVISOR, BNB_CHAIN_EID,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, BytesN, Env, String,
};

#[test]
fn test_peg_bridge_custom_sac_admin_and_18_to_7_decimal_conversion() {
    let env = Env::default();
    env.mock_all_auths();

    let stream_contract = env.register_contract(None, StellarStreamContract);
    let peg_bridge = env.register_contract(None, PegBridgeContract);
    let peg_client = PegBridgeContractClient::new(&env, &peg_bridge);

    // 1. Deploy classic STX asset SAC with classic issuer, then transfer SAC admin to peg_bridge FIRST
    let classic_issuer = Address::generate(&env);
    let stx_sac = env
        .register_stellar_asset_contract_v2(classic_issuer.clone())
        .address();
    let stx_admin_client = token::StellarAssetClient::new(&env, &stx_sac);
    stx_admin_client.set_admin(&peg_bridge);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    peg_client.initialize_peg_bridge(&admin, &relayer, &stx_sac, &stream_contract);

    // 2. Confirm classic issuer is locked (masterWeight = 0) SECOND
    peg_client.confirm_issuer_locked(&admin);

    // 3. Test 18-decimal BNB BEP-20 -> 7-decimal Stellar SAC conversion with sub-stroop dust
    // 250.123456789123456789 STX on BNB Chain = 250_123456789123456789 (18 dec)
    // -> 250_1234567 (7 dec Stellar stroops) + 89_123_456_789 (18 dec dust remainder)
    let amount_bnb_18: i128 = 250_123_456_789_123_456_789;
    let quote = peg_client.quote_bnb18_to_stellar7(&amount_bnb_18);
    assert_eq!(quote.amount_stellar_7, 250_1234567);
    assert_eq!(quote.dust_remainder_18, 89_123_456_789);
    assert_eq!(quote.source_eid, BNB_CHAIN_EID);

    // 4. Execute inbound 1:1 peg mint from BNB Chain to Stellar recipient wallet
    let recipient = Address::generate(&env);
    let msg_id = BytesN::from_array(&env, &[7u8; 32]);
    let bnb_sender = String::from_str(&env, "0x71C7656EC7ab88b098defB751B7401B5f6d8976F");

    let minted_7 =
        peg_client.mint_pegged_from_bnb(&relayer, &msg_id, &bnb_sender, &recipient, &amount_bnb_18);
    assert_eq!(minted_7, 250_1234567);
    assert!(peg_client.is_message_processed(&msg_id));

    let stx_token = token::Client::new(&env, &stx_sac);
    assert_eq!(stx_token.balance(&recipient), 250_1234567);

    // 5. Execute outbound burn back to BNB Chain (50 STX = 50_0000000 -> 50 * 10^18 on BNB)
    let burned_bnb_18 =
        peg_client.burn_for_bnb_redemption(&recipient, &bnb_sender, &50_0000000_i128);
    assert_eq!(burned_bnb_18, 50_0000000_i128 * BNB18_TO_STELLAR7_DIVISOR);
    assert_eq!(stx_token.balance(&recipient), 200_1234567);

    let stats = peg_client.get_peg_stats();
    assert_eq!(stats.total_minted_7, 250_1234567);
    assert_eq!(stats.total_burned_7, 50_0000000);
    assert_eq!(stats.active_circulating_peg_7, 200_1234567);
    assert!(stats.issuer_locked);
}

#[test]
#[should_panic]
fn test_peg_bridge_blocks_replay_of_processed_message() {
    let env = Env::default();
    env.mock_all_auths();

    let stream_contract = env.register_contract(None, StellarStreamContract);
    let peg_bridge = env.register_contract(None, PegBridgeContract);
    let peg_client = PegBridgeContractClient::new(&env, &peg_bridge);

    let classic_issuer = Address::generate(&env);
    let stx_sac = env
        .register_stellar_asset_contract_v2(classic_issuer)
        .address();
    token::StellarAssetClient::new(&env, &stx_sac).set_admin(&peg_bridge);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    peg_client.initialize_peg_bridge(&admin, &relayer, &stx_sac, &stream_contract);
    peg_client.confirm_issuer_locked(&admin);

    let recipient = Address::generate(&env);
    let msg_id = BytesN::from_array(&env, &[9u8; 32]);
    let bnb_sender = String::from_str(&env, "0x71C7656EC7ab88b098defB751B7401B5f6d8976F");
    let amount_bnb_18: i128 = 100 * BNB18_TO_STELLAR7_DIVISOR;

    peg_client.mint_pegged_from_bnb(&relayer, &msg_id, &bnb_sender, &recipient, &amount_bnb_18);
    // Replaying the exact same message_id must panic via persistent nullifier guard
    peg_client.mint_pegged_from_bnb(&relayer, &msg_id, &bnb_sender, &recipient, &amount_bnb_18);
}

#[test]
fn test_atomic_mint_and_stream_from_bnb() {
    let env = Env::default();
    env.mock_all_auths();

    let stream_contract = env.register_contract(None, StellarStreamContract);
    let peg_bridge = env.register_contract(None, PegBridgeContract);
    let stream_client = StellarStreamContractClient::new(&env, &stream_contract);
    let peg_client = PegBridgeContractClient::new(&env, &peg_bridge);

    let classic_issuer = Address::generate(&env);
    let stx_sac = env
        .register_stellar_asset_contract_v2(classic_issuer)
        .address();
    token::StellarAssetClient::new(&env, &stx_sac).set_admin(&peg_bridge);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    peg_client.initialize_peg_bridge(&admin, &relayer, &stx_sac, &stream_contract);
    peg_client.confirm_issuer_locked(&admin);

    let employer = Address::generate(&env);
    let contractor = Address::generate(&env);
    let msg_id = BytesN::from_array(&env, &[42u8; 32]);
    let bnb_sender = String::from_str(&env, "0x71C7656EC7ab88b098defB751B7401B5f6d8976F");

    // Bridge 1,000 STX (18-dec on BNB) -> 1,000.0000000 STX (7-dec on Stellar) and stream over 1,000s
    let amount_bnb_18: i128 = 1_000_0000000 * BNB18_TO_STELLAR7_DIVISOR;
    let stream_id = peg_client.mint_and_stream_from_bnb(
        &relayer,
        &msg_id,
        &bnb_sender,
        &employer,
        &contractor,
        &amount_bnb_18,
        &1_000,
    );

    // Advance 500s (50% vested) and claim 500 STX into contractor's Stellar wallet
    env.ledger().with_mut(|l| l.timestamp = 500);
    let claimed = stream_client.claim(&stream_id, &contractor, &500_0000000);
    assert_eq!(claimed, 500_0000000);
    assert_eq!(
        token::Client::new(&env, &stx_sac).balance(&contractor),
        500_0000000
    );
}
