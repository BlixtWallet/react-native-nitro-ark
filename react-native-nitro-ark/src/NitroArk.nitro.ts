import type { HybridObject } from 'react-native-nitro-modules';

// --- Interfaces matching C structs ---

export type BarkRefreshModeType =
  | 'DefaultThreshold'
  | 'ThresholdBlocks'
  | 'ThresholdHours'
  | 'Counterparty'
  | 'All'
  | 'Specific';

// Note: BarkError is handled via Promise rejection, not exposed directly.

export interface BarkConfigOpts {
  asp?: string;
  esplora?: string;
  bitcoind?: string;
  bitcoind_cookie?: string;
  bitcoind_user?: string;
  bitcoind_pass?: string;
  vtxo_refresh_expiry_threshold?: number;
  fallback_fee_rate?: number;
}

export interface BarkCreateOpts {
  force?: boolean;
  regtest?: boolean;
  signet?: boolean;
  bitcoin?: boolean;
  mnemonic: string;
  birthday_height?: number; // uint64_t might exceed safe integer, but JS number is often used. Be mindful.
  config?: BarkConfigOpts;
}

export interface BarkBalance {
  onchain: number; // uint64_t -> number
  offchain: number; // uint64_t -> number
  pending_exit: number; // uint64_t -> number
}

export interface BarkRefreshOpts {
  mode_type: BarkRefreshModeType;
  threshold_value?: number; // uint32_t -> number (Only relevant for Threshold modes)
  specific_vtxo_ids?: string[]; // const char *const * -> string[] (Only relevant for Specific mode)
}

// Helper interface for sendManyOnchain
export interface BarkSendManyOutput {
  destination: string;
  amountSat: number; // uint64_t -> number
}

// --- Nitro Module Interface ---

export interface NitroArk extends HybridObject<{ ios: 'c++'; android: 'c++' }> {
  // --- Management ---
  createMnemonic(): Promise<string>;
  createWallet(datadir: string, opts: BarkCreateOpts): Promise<void>; // Returns void on success, throws on error

  // --- Wallet Info ---
  getBalance(
    datadir: string,
    no_sync: boolean,
    mnemonic: string
  ): Promise<BarkBalance>;
  getOnchainAddress(datadir: string, mnemonic: string): Promise<string>;
  getOnchainUtxos(
    datadir: string,
    mnemonic: string,
    no_sync: boolean
  ): Promise<string>; // Returns JSON string
  getVtxoPubkey(datadir: string, mnemonic: string): Promise<string>;
  getVtxos(
    datadir: string,
    mnemonic: string,
    no_sync: boolean
  ): Promise<string>; // Returns JSON string

  // --- Onchain Operations ---
  sendOnchain(
    datadir: string,
    mnemonic: string,
    destination: string,
    amountSat: number,
    no_sync: boolean
  ): Promise<string>; // Returns txid
  drainOnchain(
    datadir: string,
    mnemonic: string,
    destination: string,
    no_sync: boolean
  ): Promise<string>; // Returns txid
  sendManyOnchain(
    datadir: string,
    mnemonic: string,
    outputs: BarkSendManyOutput[],
    no_sync: boolean
  ): Promise<string>; // Returns txid

  // --- Ark Operations ---
  refreshVtxos(
    datadir: string,
    mnemonic: string,
    refreshOpts: BarkRefreshOpts,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status
  boardAmount(
    datadir: string,
    mnemonic: string,
    amountSat: number,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status
  boardAll(
    datadir: string,
    mnemonic: string,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status
  send(
    datadir: string,
    mnemonic: string,
    destination: string,
    amountSat: number,
    comment: string | null,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status
  sendRoundOnchain(
    datadir: string,
    mnemonic: string,
    destination: string,
    amountSat: number,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status

  // --- Lightning Operations ---
  bolt11Invoice(
    datadir: string,
    mnemonic: string,
    amountSat: number
  ): Promise<string>; // Returns invoice string
  claimBolt11Payment(
    datadir: string,
    mnemonic: string,
    bolt11: string
  ): Promise<void>; // Throws on error

  // --- Offboarding / Exiting ---
  offboardSpecific(
    datadir: string,
    mnemonic: string,
    vtxoIds: string[],
    optionalAddress: string | null,
    no_sync: boolean
  ): Promise<string>; // Returns JSON result
  offboardAll(
    datadir: string,
    mnemonic: string,
    optionalAddress: string | null,
    no_sync: boolean
  ): Promise<string>; // Returns JSON result
  exitStartSpecific(
    datadir: string,
    mnemonic: string,
    vtxoIds: string[],
    no_sync: boolean
  ): Promise<string>; // Returns JSON status
  exitStartAll(
    datadir: string,
    mnemonic: string,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status
  exitProgressOnce(datadir: string, mnemonic: string): Promise<string>; // Returns JSON status
}
