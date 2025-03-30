import type { HybridObject } from 'react-native-nitro-modules';

// Define interface types that match the C++ structs
export interface BarkConfigOpts {
  asp?: string;
  esplora?: string;
  bitcoind?: string;
  bitcoind_cookie?: string;
  bitcoind_user?: string;
  bitcoind_pass?: string;
}

export interface BarkCreateOpts {
  force?: boolean;
  regtest?: boolean;
  signet?: boolean;
  bitcoin?: boolean;
  mnemonic?: string;
  birthday_height?: number;
  config?: BarkConfigOpts;
}

export interface BarkBalance {
  onchain: number;
  offchain: number;
  pending_exit: number;
}

export interface NitroArk extends HybridObject<{ ios: 'c++'; android: 'c++' }> {
  // Create a new wallet at the specified directory
  createWallet(datadir: string, opts: BarkCreateOpts): Promise<boolean>;
  
  // Get offchain and onchain balances
  getBalance(datadir: string, no_sync: boolean): Promise<BarkBalance>;
  
  // Original multiply function
  multiply(a: number, b: number): number;
}