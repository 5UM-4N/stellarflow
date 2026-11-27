import { describe, it, expect } from 'vitest';
import { StrKey } from '@stellar/stellar-sdk';
import {
  stellarRecipientToBytes32,
  evmAddressToBytes32,
  convertBnb18ToStellar7,
  deriveStxSacAddress,
  buildStxTrustlineTxXdr,
  DEFAULT_STX_ISSUER,
  BNB_CHAIN_EID,
  STELLAR_TESTNET_EID,
} from './stxPegBridge';

describe('1:1 BNB Chain (BEP-20) <-> Stellar SAC (STX) Peg Bridge Service', () => {
  const sampleGAddress = DEFAULT_STX_ISSUER;
  const sampleContractId =
    'CCJW2RLIN4MQQ4DAJMMR3F5QPDA6QYTKXJMEVI3XOTDBTBCLBB553J74';

  it('encodes Stellar G... and C... addresses into raw 32-byte hex payloads without version/checksum', () => {
    const gBytes32 = stellarRecipientToBytes32(sampleGAddress);
    const cBytes32 = stellarRecipientToBytes32(sampleContractId);

    expect(gBytes32).toMatch(/^0x[0-9a-f]{64}$/);
    expect(cBytes32).toMatch(/^0x[0-9a-f]{64}$/);
    // All-zero Ed25519 public key decodes to 64 zero hex characters
    expect(gBytes32).toBe(`0x${'0'.repeat(64)}`);
  });

  it('left-pads 20-byte BNB Chain EVM addresses to 32-byte hex payloads', () => {
    const bnbAddr = '0x71C7656EC7ab88b098defB751B7401B5f6d8976F';
    const padded = evmAddressToBytes32(bnbAddr);
    expect(padded).toBe(
      '0x00000000000000000000000071c7656ec7ab88b098defb751b7401b5f6d8976f',
    );
  });

  it('converts 18-decimal BNB BEP-20 STX amounts to 7-decimal Stellar SAC stroops with exact dust accounting', () => {
    // 250.123456789123456789 STX on BNB Chain (18 decimals)
    const quote = convertBnb18ToStellar7('250123456789123456789');
    expect(quote.amountStellar7Stroops).toBe('2501234567');
    expect(quote.formattedStellarStx).toBe('250.1234567');
    expect(quote.dustRemainderBnb18).toBe('89123456789');
    expect(quote.sourceChainEid).toBe(BNB_CHAIN_EID);
    expect(quote.destinationChainEid).toBe(STELLAR_TESTNET_EID);
  });

  it('derives a valid deterministic C... SAC contract address for classic STX asset', () => {
    const sacId = deriveStxSacAddress(sampleGAddress);
    expect(StrKey.isValidContract(sacId)).toBe(true);
    expect(sacId.startsWith('C')).toBe(true);
    expect(sacId).toHaveLength(56);
  });

  it('builds a valid ChangeTrust transaction XDR so pegged STX appears in Stellar wallets', () => {
    const xdr = buildStxTrustlineTxXdr(sampleGAddress, sampleGAddress, '500000');
    expect(typeof xdr).toBe('string');
    expect(xdr.length).toBeGreaterThan(40);
  });
});
