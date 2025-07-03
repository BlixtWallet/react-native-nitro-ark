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
  loadWallet(datadir: string, opts: BarkCreateOpts): Promise<void>;
  closeWallet(): Promise<void>;

  // --- Wallet Info ---
  getBalance(no_sync: boolean): Promise<BarkBalance>;
  getOnchainAddress(): Promise<string>;
  getOnchainUtxos(no_sync: boolean): Promise<string>; // Returns JSON string
  getVtxoPubkey(index?: number): Promise<string>;
  getVtxos(no_sync: boolean): Promise<string>; // Returns JSON string

  // --- Onchain Operations ---
  sendOnchain(
    destination: string,
    amountSat: number,
    no_sync: boolean
  ): Promise<string>; // Returns txid
  drainOnchain(destination: string, no_sync: boolean): Promise<string>; // Returns txid
  sendManyOnchain(
    outputs: BarkSendManyOutput[],
    no_sync: boolean
  ): Promise<string>; // Returns txid

  // --- Ark Operations ---
  refreshVtxos(refreshOpts: BarkRefreshOpts, no_sync: boolean): Promise<string>; // Returns JSON status
  boardAmount(amountSat: number, no_sync: boolean): Promise<string>; // Returns JSON status
  boardAll(no_sync: boolean): Promise<string>; // Returns JSON status
  send(
    destination: string,
    amountSat: number,
    comment: string | null,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status
  sendRoundOnchain(
    destination: string,
    amountSat: number,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status

  // --- Lightning Operations ---
  bolt11Invoice(amountMsat: number): Promise<string>; // Returns invoice string
  claimBolt11Payment(bolt11: string): Promise<void>; // Throws on error

  // --- Offboarding / Exiting ---
  offboardSpecific(
    vtxoIds: string[],
    optionalAddress: string | null,
    no_sync: boolean
  ): Promise<string>; // Returns JSON result
  offboardAll(
    optionalAddress: string | null,
    no_sync: boolean
  ): Promise<string>; // Returns JSON result
  exitStartSpecific(vtxoIds: string[]): Promise<string>; // Returns JSON status
  exitStartAll(): Promise<string>; // Returns JSON status
  exitProgressOnce(): Promise<string>; // Returns JSON status
}
