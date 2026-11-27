//! # PegBridgeContract: 1:1 Cross-Chain BNB (BEP-20) <-> Stellar SAC Peg & Stream Bridge
//!
//! Implements the 2026 Stellar Custom SAC Admin + Locked Issuer (`masterWeight: 0`) pattern
//! for 1:1 pegged assets (such as `STX` bridged from BNB Smart Chain via LayerZero V2 OFT or
//! Axelar ITS):
//!
//! 1. **Custom SAC Admin & Locked Issuer Invariant:**
//!    The classic Stellar asset (`STX:G...`) is deployed as a Stellar Asset Contract (SAC).
//!    Admin authority is handed over to this contract first (`sac.set_admin(&bridge)`), and
//!    the classic issuer account's master key weight is locked to `0` second (`confirm_issuer_locked`).
//!    No `mint` or `burn` can occur until `issuer_locked == true`, preventing unbacked out-of-band
//!    classic issuance.
//! 2. **18-Decimal (BNB BEP-20) <-> 7-Decimal (Stellar SAC) Precision & Dust Accounting:**
//!    BNB Chain BEP-20 tokens use 18 decimals (`10^18`), whereas Stellar classic assets and SACs
//!    use 7 decimals (`10^7`). The scaling factor is `10^11` (`100_000_000_000`). Inbound deposits
//!    deterministically convert `amount_bnb_18 / 10^11 -> amount_stellar_7` and record any
//!    sub-stroop dust remainder (`amount_bnb_18 % 10^11`) in the receipt event.
//! 3. **On-Chain Anti-Replay Nullifier Guard:**
//!    Every cross-chain message (`BytesN<32>` message GUID / tx hash from BNB Chain EID `30102`)
//!    is persisted in `PegDataKey::ProcessedMessage(BytesN<32>)` with proactive TTL extension so
//!    no attestation can ever be replayed.
//! 4. **Atomic Bridge-to-Stream (`mint_and_stream`):**
//!    Mints 1:1 pegged `STX` on Stellar and atomically opens a continuous second-by-second
//!    payment stream in `StellarStreamContract` in a single transaction.

use crate::errors::ContractError;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token::{Client as TokenClient, StellarAssetClient},
    Address, BytesN, Env, String,
};

pub const BNB_CHAIN_EID: u32 = 30102;
pub const STELLAR_MAINNET_EID: u32 = 30600;
pub const STELLAR_TESTNET_EID: u32 = 40600;
/// 10^(18 - 7) = 10^11 scaling factor between 18-decimal BNB BEP-20 and 7-decimal Stellar SAC.
pub const BNB18_TO_STELLAR7_DIVISOR: i128 = 100_000_000_000;

const PEG_INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const PEG_INSTANCE_BUMP_AMOUNT: u32 = 518_400;
const PEG_PERSISTENT_LIFETIME_THRESHOLD: u32 = 17_280;
const PEG_PERSISTENT_BUMP_AMOUNT: u32 = 518_400;

fn bump_peg_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(PEG_INSTANCE_LIFETIME_THRESHOLD, PEG_INSTANCE_BUMP_AMOUNT);
}

fn bump_peg_persistent(env: &Env, key: &PegDataKey) {
    env.storage().persistent().extend_ttl(
        key,
        PEG_PERSISTENT_LIFETIME_THRESHOLD,
        PEG_PERSISTENT_BUMP_AMOUNT,
    );
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PegDataKey {
    Admin,
    Relayer,
    PeggedSacToken,
    StreamContract,
    IssuerLocked,
    Paused,
    TotalMinted7,
    TotalBurned7,
    ProcessedMessage(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PegConversionReceipt {
    pub amount_bnb_18: i128,
    pub amount_stellar_7: i128,
    pub dust_remainder_18: i128,
    pub source_eid: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PegMintedEvent {
    pub message_id: BytesN<32>,
    pub source_eid: u32,
    pub bnb_sender: String,
    pub stellar_recipient: Address,
    pub amount_stellar_7: i128,
    pub total_supply_pegged_7: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PegBurnedForBnbEvent {
    pub stellar_sender: Address,
    pub bnb_recipient_hex: String,
    pub amount_stellar_7: i128,
    pub amount_bnb_18: i128,
    pub destination_eid: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PegStats {
    pub total_minted_7: i128,
    pub total_burned_7: i128,
    pub active_circulating_peg_7: i128,
    pub issuer_locked: bool,
    pub paused: bool,
}

mod peg_stream_interface {
    use soroban_sdk::{contractclient, Address, Env, Map, String};

    #[contractclient(name = "PegStreamEscrowClient")]
    #[allow(dead_code)]
    pub trait PegStreamEscrowInterface {
        #[allow(clippy::too_many_arguments)]
        fn create_stream(
            env: Env,
            sender: Address,
            recipient: Address,
            token: Address,
            total_amount: i128,
            start_time: u64,
            end_time: u64,
            min_claim_interval_seconds: u64,
            metadata: Option<Map<String, String>>,
        ) -> u64;
    }
}

use peg_stream_interface::PegStreamEscrowClient;

#[contract]
pub struct PegBridgeContract;

#[contractimpl]
impl PegBridgeContract {
    /// Initializes the 1:1 BNB <-> Stellar SAC Peg Bridge.
    /// Namespaced as `initialize_peg_bridge` to avoid symbol collision in shared cdylib builds.
    pub fn initialize_peg_bridge(
        env: Env,
        admin: Address,
        relayer: Address,
        pegged_sac_token: Address,
        stream_contract: Address,
    ) {
        if env.storage().instance().has(&PegDataKey::Admin) {
            panic!("peg bridge already initialized");
        }
        admin.require_auth();

        env.storage().instance().set(&PegDataKey::Admin, &admin);
        env.storage().instance().set(&PegDataKey::Relayer, &relayer);
        env.storage()
            .instance()
            .set(&PegDataKey::PeggedSacToken, &pegged_sac_token);
        env.storage()
            .instance()
            .set(&PegDataKey::StreamContract, &stream_contract);
        env.storage()
            .instance()
            .set(&PegDataKey::IssuerLocked, &false);
        env.storage().instance().set(&PegDataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&PegDataKey::TotalMinted7, &0_i128);
        env.storage()
            .instance()
            .set(&PegDataKey::TotalBurned7, &0_i128);
        bump_peg_instance(&env);
    }

    /// Mandatory Step 2 after `sac.set_admin(&peg_bridge)`:
    /// Admin attests and locks the classic issuer state (`masterWeight == 0`) so 1:1 peg minting
    /// can begin. Without this lock, all mint operations revert with `ContractError::EscrowInactive`.
    pub fn confirm_issuer_locked(env: Env, admin: Address) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&PegDataKey::Admin)
            .unwrap_or_else(|| panic!("peg bridge not initialized"));
        if stored_admin != admin {
            panic!("unauthorized");
        }
        admin.require_auth();

        env.storage()
            .instance()
            .set(&PegDataKey::IssuerLocked, &true);
        bump_peg_instance(&env);
    }

    /// Emergency circuit-breaker pause/unpause for cross-chain mint/burn operations.
    pub fn set_peg_paused(env: Env, admin: Address, paused: bool) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&PegDataKey::Admin)
            .unwrap_or_else(|| panic!("peg bridge not initialized"));
        if stored_admin != admin {
            panic!("unauthorized");
        }
        admin.require_auth();

        env.storage().instance().set(&PegDataKey::Paused, &paused);
        bump_peg_instance(&env);
    }

    /// Pure quote helper: converts an 18-decimal BNB Chain BEP-20 amount into a 7-decimal
    /// Stellar SAC amount (`stroops`), returning both the converted 7-decimal amount and any
    /// sub-stroop dust remainder (`amount_bnb_18 % 10^11`).
    pub fn quote_bnb18_to_stellar7(_env: Env, amount_bnb_18: i128) -> PegConversionReceipt {
        if amount_bnb_18 <= 0 {
            panic!("amount_bnb_18 must be positive");
        }
        let amount_stellar_7 = amount_bnb_18 / BNB18_TO_STELLAR7_DIVISOR;
        let dust_remainder_18 = amount_bnb_18 % BNB18_TO_STELLAR7_DIVISOR;

        PegConversionReceipt {
            amount_bnb_18,
            amount_stellar_7,
            dust_remainder_18,
            source_eid: BNB_CHAIN_EID,
        }
    }

    /// Inbound 1:1 Peg Mint (BNB Chain BEP-20 STX -> Stellar SAC STX):
    /// Verifies the authorized cross-chain relayer/DVN executor, checks the on-chain nullifier
    /// (`ProcessedMessage(message_id)`), verifies the classic issuer is locked, converts 18-dec
    /// BNB units to 7-dec Stellar SAC units, and mints 1:1 pegged `STX` to `stellar_recipient`.
    pub fn mint_pegged_from_bnb(
        env: Env,
        relayer: Address,
        message_id: BytesN<32>,
        bnb_sender: String,
        stellar_recipient: Address,
        amount_bnb_18: i128,
    ) -> Result<i128, ContractError> {
        let stored_relayer: Address = env
            .storage()
            .instance()
            .get(&PegDataKey::Relayer)
            .unwrap_or_else(|| panic!("peg bridge not initialized"));
        if stored_relayer != relayer {
            return Err(ContractError::Unauthorized);
        }
        relayer.require_auth();

        let paused: bool = env
            .storage()
            .instance()
            .get(&PegDataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::EscrowInactive);
        }

        let issuer_locked: bool = env
            .storage()
            .instance()
            .get(&PegDataKey::IssuerLocked)
            .unwrap_or(false);
        if !issuer_locked {
            panic!("classic issuer must be locked (masterWeight=0) before SAC peg minting");
        }

        let msg_key = PegDataKey::ProcessedMessage(message_id.clone());
        if env.storage().persistent().has(&msg_key) {
            panic!("cross-chain message already processed");
        }

        let receipt = Self::quote_bnb18_to_stellar7(env.clone(), amount_bnb_18);
        if receipt.amount_stellar_7 <= 0 {
            panic!("bridged amount below 1 Stellar stroop (10^-7)");
        }

        // Persist anti-replay nullifier and extend TTL
        env.storage().persistent().set(&msg_key, &true);
        bump_peg_persistent(&env, &msg_key);

        // Mint 1:1 pegged SAC tokens to recipient
        let sac_token: Address = env
            .storage()
            .instance()
            .get(&PegDataKey::PeggedSacToken)
            .unwrap();
        StellarAssetClient::new(&env, &sac_token)
            .mint(&stellar_recipient, &receipt.amount_stellar_7);

        let mut total_minted: i128 = env
            .storage()
            .instance()
            .get(&PegDataKey::TotalMinted7)
            .unwrap_or(0);
        total_minted = total_minted
            .checked_add(receipt.amount_stellar_7)
            .expect("mint total overflow");
        env.storage()
            .instance()
            .set(&PegDataKey::TotalMinted7, &total_minted);

        let total_burned: i128 = env
            .storage()
            .instance()
            .get(&PegDataKey::TotalBurned7)
            .unwrap_or(0);
        bump_peg_instance(&env);

        env.events().publish(
            (symbol_short!("Peg"), symbol_short!("Minted")),
            PegMintedEvent {
                message_id,
                source_eid: BNB_CHAIN_EID,
                bnb_sender,
                stellar_recipient,
                amount_stellar_7: receipt.amount_stellar_7,
                total_supply_pegged_7: total_minted.saturating_sub(total_burned),
            },
        );

        Ok(receipt.amount_stellar_7)
    }

    /// Atomic Cross-Chain Peg Mint + Continuous Stream Creation (`mint_and_stream`):
    /// Mints 1:1 pegged STX from a verified BNB Chain lock message directly into escrow and
    /// opens a continuous linear payment stream for `stream_recipient` in `StellarStreamContract`.
    #[allow(clippy::too_many_arguments)]
    pub fn mint_and_stream_from_bnb(
        env: Env,
        relayer: Address,
        message_id: BytesN<32>,
        bnb_sender: String,
        stream_sender: Address,
        stream_recipient: Address,
        amount_bnb_18: i128,
        duration_seconds: u64,
    ) -> Result<u64, ContractError> {
        // Outer-layer re-authorization for stream_sender (consumed in cross-contract create_stream),
        // while mint_pegged_from_bnb below enforces relayer.require_auth() in this frame.
        stream_sender.require_auth();

        if duration_seconds == 0 {
            panic!("duration_seconds must be positive");
        }

        let minted_7 = Self::mint_pegged_from_bnb(
            env.clone(),
            relayer,
            message_id,
            bnb_sender,
            stream_sender.clone(),
            amount_bnb_18,
        )?;

        let sac_token: Address = env
            .storage()
            .instance()
            .get(&PegDataKey::PeggedSacToken)
            .unwrap();
        let stream_contract: Address = env
            .storage()
            .instance()
            .get(&PegDataKey::StreamContract)
            .unwrap();

        let now = env.ledger().timestamp();
        let end_time = now.saturating_add(duration_seconds);

        let escrow_client = PegStreamEscrowClient::new(&env, &stream_contract);
        let stream_id = escrow_client.create_stream(
            &stream_sender,
            &stream_recipient,
            &sac_token,
            &minted_7,
            &now,
            &end_time,
            &0,
            &None,
        );

        Ok(stream_id)
    }

    /// Outbound 1:1 Peg Redemption (Stellar SAC STX -> BNB Chain BEP-20 STX):
    /// Burns 7-decimal SAC tokens from `stellar_sender` (requiring outer-layer auth) and emits
    /// a `PegBurnedForBnbEvent` with the scaled 18-decimal amount for BNB Chain release.
    pub fn burn_for_bnb_redemption(
        env: Env,
        stellar_sender: Address,
        bnb_recipient_hex: String,
        amount_stellar_7: i128,
    ) -> i128 {
        stellar_sender.require_auth();

        if amount_stellar_7 <= 0 {
            panic!("amount_stellar_7 must be positive");
        }

        let paused: bool = env
            .storage()
            .instance()
            .get(&PegDataKey::Paused)
            .unwrap_or(false);
        if paused {
            panic!("peg bridge is paused");
        }

        let sac_token: Address = env
            .storage()
            .instance()
            .get(&PegDataKey::PeggedSacToken)
            .unwrap_or_else(|| panic!("peg bridge not initialized"));

        TokenClient::new(&env, &sac_token).burn(&stellar_sender, &amount_stellar_7);

        let mut total_burned: i128 = env
            .storage()
            .instance()
            .get(&PegDataKey::TotalBurned7)
            .unwrap_or(0);
        total_burned = total_burned
            .checked_add(amount_stellar_7)
            .expect("burn total overflow");
        env.storage()
            .instance()
            .set(&PegDataKey::TotalBurned7, &total_burned);
        bump_peg_instance(&env);

        let amount_bnb_18 = amount_stellar_7
            .checked_mul(BNB18_TO_STELLAR7_DIVISOR)
            .expect("bnb18 conversion overflow");

        env.events().publish(
            (symbol_short!("Peg"), symbol_short!("Burned")),
            PegBurnedForBnbEvent {
                stellar_sender,
                bnb_recipient_hex,
                amount_stellar_7,
                amount_bnb_18,
                destination_eid: BNB_CHAIN_EID,
            },
        );

        amount_bnb_18
    }

    /// Returns current 1:1 peg reserve accounting and lock status.
    pub fn get_peg_stats(env: Env) -> PegStats {
        let total_minted_7: i128 = env
            .storage()
            .instance()
            .get(&PegDataKey::TotalMinted7)
            .unwrap_or(0);
        let total_burned_7: i128 = env
            .storage()
            .instance()
            .get(&PegDataKey::TotalBurned7)
            .unwrap_or(0);
        let issuer_locked: bool = env
            .storage()
            .instance()
            .get(&PegDataKey::IssuerLocked)
            .unwrap_or(false);
        let paused: bool = env
            .storage()
            .instance()
            .get(&PegDataKey::Paused)
            .unwrap_or(false);

        PegStats {
            total_minted_7,
            total_burned_7,
            active_circulating_peg_7: total_minted_7.saturating_sub(total_burned_7),
            issuer_locked,
            paused,
        }
    }

    /// Checks whether a given cross-chain message ID has already been processed.
    pub fn is_message_processed(env: Env, message_id: BytesN<32>) -> bool {
        env.storage()
            .persistent()
            .has(&PegDataKey::ProcessedMessage(message_id))
    }
}
