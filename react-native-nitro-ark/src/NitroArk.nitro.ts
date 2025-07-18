import type { HybridObject } from 'react-native-nitro-modules';

// --- Interfaces matching C structs ---

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
  birthday_height?: number;
  config?: BarkConfigOpts;
}

export interface BarkArkInfo {
  network: string;
  asp_pubkey: string;
  round_interval_secs: number; // u64
  vtxo_exit_delta: number; // u16
  vtxo_expiry_delta: number; // u16
  htlc_expiry_delta: number; // u16
  max_vtxo_amount_sat: number; // u64
}

// Helper interface for sendManyOnchain
export interface BarkSendManyOutput {
  destination: string;
  amountSat: number; // uint64_t -> number
}

export type PaymentTypes = 'Bolt11' | 'Lnurl' | 'Arkoor' | 'Onchain';

export interface BarkVtxo {
  amount: number; // u64
  expiry_height: number; // u32
  exit_delta: number; // u16
  anchor_point: string;
}

export interface ArkoorPaymentResult {
  amount_sat: number; // u64
  destination_pubkey: string;
  payment_type: PaymentTypes;
  vtxos: BarkVtxo[];
}

export interface Bolt11PaymentResult {
  bolt11_invoice: string;
  preimage: string;
  payment_type: PaymentTypes;
}

export interface LnurlPaymentResult {
  lnurl: string;
  bolt11_invoice: string;
  preimage: string;
  payment_type: PaymentTypes;
}

// --- Nitro Module Interface ---

export interface NitroArk extends HybridObject<{ ios: 'c++'; android: 'c++' }> {
  // --- Management ---
  createMnemonic(): Promise<string>;
  loadWallet(datadir: string, opts: BarkCreateOpts): Promise<void>;
  closeWallet(): Promise<void>;
  isWalletLoaded(): Promise<boolean>;
  persistConfig(opts: BarkConfigOpts): Promise<void>;
  maintenance(): Promise<void>;
  sync(): Promise<void>;
  syncArk(): Promise<void>;
  syncRounds(): Promise<void>;

  // --- Wallet Info ---
  getArkInfo(): Promise<BarkArkInfo>;
  onchainBalance(): Promise<number>;
  offchainBalance(): Promise<number>;
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

  // --- Ark & Lightning Payments ---
  boardAmount(amountSat: number): Promise<string>; // Returns JSON status
  boardAll(): Promise<string>; // Returns JSON status
  sendArkoorPayment(
    destination: string,
    amountSat: number
  ): Promise<ArkoorPaymentResult>;
  sendBolt11Payment(
    destination: string,
    amountSat?: number
  ): Promise<Bolt11PaymentResult>;
  sendLnaddr(
    addr: string,
    amountSat: number,
    comment: string
  ): Promise<LnurlPaymentResult>;
  sendRoundOnchain(
    destination: string,
    amountSat: number,
    no_sync: boolean
  ): Promise<string>; // Returns JSON status

  // --- Lightning Invoicing ---
  bolt11Invoice(amountMsat: number): Promise<string>; // Returns invoice string
  claimBolt11Payment(bolt11: string): Promise<void>; // Throws on error

  // --- Offboarding / Exiting ---
  offboardSpecific(
    vtxoIds: string[],
    destinationAddress: string
  ): Promise<string>; // Returns JSON result
  offboardAll(destinationAddress: string): Promise<string>; // Returns JSON result
  exitStartSpecific(vtxoIds: string[]): Promise<string>;
  exitStartAll(): Promise<string>;
  exitProgressOnce(): Promise<string>;
}
