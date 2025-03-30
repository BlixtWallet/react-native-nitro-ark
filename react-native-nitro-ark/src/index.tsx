import { NitroModules } from 'react-native-nitro-modules';
import type { NitroArk } from './NitroArk.nitro';

const NitroArkHybridObject =
  NitroModules.createHybridObject<NitroArk>('NitroArk');

export function multiply(a: number, b: number): number {
  return NitroArkHybridObject.multiply(a, b);
}
