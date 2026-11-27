/**
 * 1:1 BNB Chain (BEP-20) <-> Stellar SAC (STX) Peg Bridge & Trustline Service
 *
 * Implements the 2026 Stellar Cross-Chain & Asset Interop Standards (`assets` + `cross-chain`):
 * 1. Classic Stellar Asset (`STX:<ISSUER>`) + Trustline (`Operation.changeTrust`) so 1:1 pegged
 *    STX appears natively in any Stellar wallet (Freighter, LOBSTR, xBull).
 * 2. Deterministic Stellar Asset Contract (SAC) derivation (`asset.contractId(networkPassphrase)`)
 *    exposing the SEP-41 smart contract token interface for continuous streaming in StellarFlow.
 * 3. LayerZero V2 OFT / Axelar ITS 32-byte recipient payload encoding (`stellarRecipientToBytes32`)
 *    stripping the strkey version byte and CRC16 checksum (`StrKey.decodeEd25519PublicKey` /
 *    `StrKey.decodeContract`).
 * 4. 18-decimal (BNB Chain BEP-20) <-> 7-decimal (Stellar SAC) dust-safe integer conversion
 *    (`10^11` divisor).
 */

import {
  Account,
  Asset,
  Operation,
  StrKey,
  TransactionBuilder,
} from '@stellar/stellar-sdk';

export const BNB_CHAIN_EID = 30102;
export const STELLAR_MAINNET_EID = 30600;
export const STELLAR_TESTNET_EID = 40600;
export const BNB18_TO_STELLAR7_SCALE = 100_000_000_000n; // 10^(18 - 7)

export const STX_ASSET_CODE = 'STX';
export const DEFAULT_STX_ISSUER =
  'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF';
export const TESTNET_PASSPHRASE = 'Test SDF Network ; September 2015';

export interface StxPegConversionQuote {
  amountBnb18: string;
  amountStellar7Stroops: string;
  formattedStellarStx: string;
  dustRemainderBnb18: string;
  sourceChainEid: number;
  destinationChainEid: number;
}

/**
 * Encodes a Stellar `G...` account or `C...` contract strkey into the exact raw
 * 32-byte hex payload (`0x` + 64 hex chars) required by LayerZero V2 OFT and
 * Axelar ITS cross-chain inbound messages from BNB Smart Chain.
 *
 * Never sends the ASCII strkey, version byte, or 2-byte checksum.
 */
export function stellarRecipientToBytes32(strkey: string): `0x${string}` {
  let raw: Uint8Array;
  if (StrKey.isValidContract(strkey)) {
    raw = StrKey.decodeContract(strkey);
  } else if (StrKey.isValidEd25519PublicKey(strkey)) {
    raw = StrKey.decodeEd25519PublicKey(strkey);
  } else {
    throw new Error(`Invalid Stellar G... or C... strkey: ${strkey}`);
  }

  let hex = '';
  for (let i = 0; i < raw.length; i++) {
    hex += raw[i].toString(16).padStart(2, '0');
  }
  return `0x${hex}`;
}

/**
 * Encodes a 20-byte BNB Smart Chain EVM address (`0x` + 40 hex chars) into a
 * left-padded 32-byte hex payload (`0x` + 64 hex chars) for outbound Stellar -> BNB redemptions.
 */
export function evmAddressToBytes32(evmAddress: string): `0x${string}` {
  const cleaned = evmAddress.trim().replace(/^0x/i, '').toLowerCase();
  if (!/^[0-9a-f]{40}$/.test(cleaned)) {
    throw new Error(`Invalid 20-byte BNB Chain EVM address: ${evmAddress}`);
  }
  return `0x${cleaned.padStart(64, '0')}`;
}

/**
 * Converts an 18-decimal BNB Chain BEP-20 `STX` amount into 7-decimal Stellar
 * SAC stroops (`10^-7`), returning the exact Stellar credit and any sub-stroop dust remainder.
 */
export function convertBnb18ToStellar7(
  amountBnb18: bigint | string,
  destinationEid: number = STELLAR_TESTNET_EID,
): StxPegConversionQuote {
  const raw18 = typeof amountBnb18 === 'bigint' ? amountBnb18 : BigInt(amountBnb18);
  if (raw18 <= 0n) {
    throw new Error('BNB Chain 18-decimal amount must be positive.');
  }

  const stroops7 = raw18 / BNB18_TO_STELLAR7_SCALE;
  const dust18 = raw18 % BNB18_TO_STELLAR7_SCALE;

  const whole = stroops7 / 10_000_000n;
  const frac = (stroops7 % 10_000_000n).toString().padStart(7, '0');

  return {
    amountBnb18: raw18.toString(),
    amountStellar7Stroops: stroops7.toString(),
    formattedStellarStx: `${whole.toString()}.${frac}`,
    dustRemainderBnb18: dust18.toString(),
    sourceChainEid: BNB_CHAIN_EID,
    destinationChainEid: destinationEid,
  };
}

/**
 * Derives the deterministic SEP-41 Stellar Asset Contract (SAC) `C...` address
 * for the classic `STX:<issuer>` pegged asset on the specified Stellar network.
 */
export function deriveStxSacAddress(
  issuerPublicKey: string = DEFAULT_STX_ISSUER,
  networkPassphrase: string = TESTNET_PASSPHRASE,
): string {
  if (!StrKey.isValidEd25519PublicKey(issuerPublicKey)) {
    throw new Error('Invalid Stellar issuer public key for STX asset.');
  }
  const stxAsset = new Asset(STX_ASSET_CODE, issuerPublicKey);
  return stxAsset.contractId(networkPassphrase);
}

/**
 * Builds a classic `ChangeTrust` transaction XDR (`Operation.changeTrust`) for
 * `STX:<issuer>`. A classic `G...` Stellar wallet MUST execute this trustline
 * transaction before receiving 1:1 pegged STX from BNB Chain so the token
 * appears in the user's Stellar wallet balance list.
 */
export function buildStxTrustlineTxXdr(
  userPublicKey: string,
  issuerPublicKey: string = DEFAULT_STX_ISSUER,
  limit: string = '1000000000',
  networkPassphrase: string = TESTNET_PASSPHRASE,
): string {
  if (!StrKey.isValidEd25519PublicKey(userPublicKey)) {
    throw new Error('Recipient must be a valid G... Stellar account public key.');
  }
  if (!StrKey.isValidEd25519PublicKey(issuerPublicKey)) {
    throw new Error('Issuer must be a valid G... Stellar account public key.');
  }

  const account = new Account(userPublicKey, '1');
  const stxAsset = new Asset(STX_ASSET_CODE, issuerPublicKey);

  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase,
  })
    .addOperation(
      Operation.changeTrust({
        asset: stxAsset,
        limit,
      }),
    )
    .setTimeout(180)
    .build();

  return tx.toXDR();
}
