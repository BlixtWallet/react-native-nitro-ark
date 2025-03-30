import { NitroModules } from 'react-native-nitro-modules';
import type { NitroArk, BarkCreateOpts, BarkBalance } from './NitroArk.nitro';

export const NitroArkHybridObject = NitroModules.createHybridObject<NitroArk>('NitroArk');

export function multiply(a: number, b: number): number {
  return NitroArkHybridObject.multiply(a, b);
}

export function createWallet(datadir: string, opts: BarkCreateOpts): Promise<boolean> {
  return NitroArkHybridObject.createWallet(datadir, opts);
}

export function getBalance(datadir: string, no_sync: boolean = false): Promise<BarkBalance> {
  return NitroArkHybridObject.getBalance(datadir, no_sync);
}

// Re-export types
export type { BarkCreateOpts, BarkConfigOpts, BarkBalance } from './NitroArk.nitro';