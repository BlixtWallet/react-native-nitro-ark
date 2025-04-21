import { NitroModules } from 'react-native-nitro-modules';
import type {
  NitroArk,
  BarkCreateOpts,
  BarkBalance,
  BarkConfigOpts,
  BarkRefreshModeType,
  BarkRefreshOpts,
  BarkSendManyOutput,
} from './NitroArk.nitro';

// Create the hybrid object instance
export const NitroArkHybridObject =
  NitroModules.createHybridObject<NitroArk>('NitroArk');

// --- Management ---

/**
 * Creates a new BIP39 mnemonic phrase.
 * @returns A promise resolving to the mnemonic string.
 */
export function createMnemonic(): Promise<string> {
  return NitroArkHybridObject.createMnemonic();
}

/**
 * Creates a new wallet at the specified directory.
 * @param datadir Path to the data directory.
 * @param opts Creation options.
 * @returns A promise that resolves on success or rejects on error.
 */
export function createWallet(
  datadir: string,
  opts: BarkCreateOpts
): Promise<void> {
  return NitroArkHybridObject.createWallet(datadir, opts);
}

// --- Wallet Info ---

/**
 * Gets the offchain and onchain balances.
 * @param datadir Path to the data directory.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @param mnemonic The wallet mnemonic phrase.
 * @returns A promise resolving to the BarkBalance object.
 */
export function getBalance(
  datadir: string,
  mnemonic: string,
  no_sync: boolean = false
): Promise<BarkBalance> {
  // Pass mnemonic correctly, adjusted default position for optional no_sync
  return NitroArkHybridObject.getBalance(datadir, no_sync, mnemonic);
}

/**
 * Gets a fresh onchain address.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @returns A promise resolving to the Bitcoin address string.
 */
export function getOnchainAddress(
  datadir: string,
  mnemonic: string
): Promise<string> {
  return NitroArkHybridObject.getOnchainAddress(datadir, mnemonic);
}

/**
 * Gets the list of onchain UTXOs as a JSON string.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to the JSON string of UTXOs.
 */
export function getOnchainUtxos(
  datadir: string,
  mnemonic: string,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.getOnchainUtxos(datadir, mnemonic, no_sync);
}

/**
 * Gets the wallet's VTXO public key (hex string).
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @returns A promise resolving to the hex-encoded public key string.
 */
export function getVtxoPubkey(
  datadir: string,
  mnemonic: string
): Promise<string> {
  return NitroArkHybridObject.getVtxoPubkey(datadir, mnemonic);
}

/**
 * Gets the list of VTXOs as a JSON string.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to the JSON string of VTXOs.
 */
export function getVtxos(
  datadir: string,
  mnemonic: string,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.getVtxos(datadir, mnemonic, no_sync);
}

// --- Onchain Operations ---

/**
 * Sends funds using the onchain wallet.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param destination The destination Bitcoin address.
 * @param amountSat The amount to send in satoshis.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to the transaction ID string.
 */
export function sendOnchain(
  datadir: string,
  mnemonic: string,
  destination: string,
  amountSat: number,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.sendOnchain(
    datadir,
    mnemonic,
    destination,
    amountSat,
    no_sync
  );
}

/**
 * Sends all funds from the onchain wallet to a destination address.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param destination The destination Bitcoin address.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to the transaction ID string.
 */
export function drainOnchain(
  datadir: string,
  mnemonic: string,
  destination: string,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.drainOnchain(
    datadir,
    mnemonic,
    destination,
    no_sync
  );
}

/**
 * Sends funds to multiple recipients using the onchain wallet.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param outputs An array of objects containing destination address and amountSat.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to the transaction ID string.
 */
export function sendManyOnchain(
  datadir: string,
  mnemonic: string,
  outputs: BarkSendManyOutput[],
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.sendManyOnchain(
    datadir,
    mnemonic,
    outputs,
    no_sync
  );
}

// --- Ark Operations ---

/**
 * Refreshes VTXOs based on specified criteria.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param refreshOpts Options specifying which VTXOs to refresh.
 *                    `mode_type` should be one of: 'DefaultThreshold', 'ThresholdBlocks', 'ThresholdHours', 'Counterparty', 'All', 'Specific'.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 * @example
 * // Refresh using default threshold
 * refreshVtxos(datadir, mnemonic, { mode_type: 'DefaultThreshold' });
 * // Refresh specific VTXOs
 * refreshVtxos(datadir, mnemonic, { mode_type: 'Specific', specific_vtxo_ids: ['vtxo_id_1', 'vtxo_id_2'] });
 * // Refresh if older than 10 blocks
 * refreshVtxos(datadir, mnemonic, { mode_type: 'ThresholdBlocks', threshold_value: 10 });
 */
export function refreshVtxos(
  datadir: string,
  mnemonic: string,
  refreshOpts: BarkRefreshOpts,
  no_sync: boolean = false
): Promise<string> {
  // Ensure mode_type is provided (should be handled by TS type system)
  if (!refreshOpts.mode_type) {
    return Promise.reject(
      new Error('refreshVtxos requires refreshOpts.mode_type')
    );
  }
  // Additional validation for specific modes could be added here if desired
  if (
    refreshOpts.mode_type === 'Specific' &&
    (!refreshOpts.specific_vtxo_ids ||
      refreshOpts.specific_vtxo_ids.length === 0)
  ) {
    return Promise.reject(
      new Error(
        "refreshVtxos with mode_type 'Specific' requires non-empty specific_vtxo_ids array"
      )
    );
  }
  if (
    (refreshOpts.mode_type === 'ThresholdBlocks' ||
      refreshOpts.mode_type === 'ThresholdHours') &&
    (refreshOpts.threshold_value === undefined ||
      refreshOpts.threshold_value <= 0)
  ) {
    return Promise.reject(
      new Error(
        `refreshVtxos with mode_type '${refreshOpts.mode_type}' requires a positive threshold_value`
      )
    );
  }
  return NitroArkHybridObject.refreshVtxos(
    datadir,
    mnemonic,
    refreshOpts,
    no_sync
  );
}

/**
 * Boards a specific amount from the onchain wallet into Ark.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param amountSat The amount in satoshis to board.
 * @param no_sync Whether to skip syncing the onchain wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function boardAmount(
  datadir: string,
  mnemonic: string,
  amountSat: number,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.boardAmount(
    datadir,
    mnemonic,
    amountSat,
    no_sync
  );
}

/**
 * Boards all available funds from the onchain wallet into Ark.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param no_sync Whether to skip syncing the onchain wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function boardAll(
  datadir: string,
  mnemonic: string,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.boardAll(datadir, mnemonic, no_sync);
}

/**
 * Sends funds offchain using Ark VTXOs.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param destination Ark address (VTXO pubkey) or onchain Bitcoin address.
 * @param amountSat The amount in satoshis to send.
 * @param comment Optional comment (can be null).
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function send(
  datadir: string,
  mnemonic: string,
  destination: string,
  amountSat: number,
  comment: string | null = null,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.send(
    datadir,
    mnemonic,
    destination,
    amountSat,
    comment,
    no_sync
  );
}

/**
 * Sends an onchain payment via an Ark round.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param destination The destination Bitcoin address.
 * @param amountSat The amount in satoshis to send.
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function sendRoundOnchain(
  datadir: string,
  mnemonic: string,
  destination: string,
  amountSat: number,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.sendRoundOnchain(
    datadir,
    mnemonic,
    destination,
    amountSat,
    no_sync
  );
}

// --- Offboarding / Exiting ---

/**
 * Offboards specific VTXOs to an optional onchain address.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param vtxoIds Array of VtxoId strings to offboard.
 * @param optionalAddress Optional destination Bitcoin address (null if sending to internal wallet).
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to a JSON result string.
 */
export function offboardSpecific(
  datadir: string,
  mnemonic: string,
  vtxoIds: string[],
  optionalAddress: string | null = null,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.offboardSpecific(
    datadir,
    mnemonic,
    vtxoIds,
    optionalAddress,
    no_sync
  );
}

/**
 * Offboards all VTXOs to an optional onchain address.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param optionalAddress Optional destination Bitcoin address (null if sending to internal wallet).
 * @param no_sync Whether to skip syncing the wallet. Defaults to false.
 * @returns A promise resolving to a JSON result string.
 */
export function offboardAll(
  datadir: string,
  mnemonic: string,
  optionalAddress: string | null = null,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.offboardAll(
    datadir,
    mnemonic,
    optionalAddress,
    no_sync
  );
}

/**
 * Starts the exit process for specific VTXOs.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param vtxoIds Array of VtxoId strings to start exiting.
 * @param no_sync Whether to skip syncing the wallet (Note: This might depend on potential C header updates). Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function exitStartSpecific(
  datadir: string,
  mnemonic: string,
  vtxoIds: string[],
  no_sync: boolean = false
): Promise<string> {
  // Passing no_sync, aligning with the TS/C++ interface definition, even if C header might differ
  return NitroArkHybridObject.exitStartSpecific(
    datadir,
    mnemonic,
    vtxoIds,
    no_sync
  );
}

/**
 * Starts the exit process for all VTXOs in the wallet.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @param no_sync Whether to skip syncing the wallet (Note: This might depend on potential C header updates). Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function exitStartAll(
  datadir: string,
  mnemonic: string,
  no_sync: boolean = false
): Promise<string> {
  // Passing no_sync, aligning with the TS/C++ interface definition, even if C header might differ
  return NitroArkHybridObject.exitStartAll(datadir, mnemonic, no_sync);
}

/**
 * Progresses the exit process once and returns the current status.
 * @param datadir Path to the data directory.
 * @param mnemonic The wallet mnemonic phrase.
 * @returns A promise resolving to a JSON status string.
 */
export function exitProgressOnce(
  datadir: string,
  mnemonic: string
): Promise<string> {
  return NitroArkHybridObject.exitProgressOnce(datadir, mnemonic);
}

// --- Original Function ---

/**
 * Multiplies two numbers (example function).
 * @param a First number.
 * @param b Second number.
 * @returns The result of the multiplication.
 */
export function multiply(a: number, b: number): number {
  // This is synchronous as defined in NitroArk.nitro.ts
  return NitroArkHybridObject.multiply(a, b);
}

// --- Re-export types and enums ---
export type {
  BarkCreateOpts,
  BarkConfigOpts,
  BarkBalance,
  BarkRefreshOpts,
  BarkRefreshModeType,
  BarkSendManyOutput,
} from './NitroArk.nitro';
