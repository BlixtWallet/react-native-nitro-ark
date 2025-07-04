import { NitroModules } from 'react-native-nitro-modules';
import type {
  NitroArk,
  BarkCreateOpts,
  BarkBalance,
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
 * Loads an existing wallet or creates a new one at the specified directory.
 * Once loaded, the wallet state is managed internally.
 * @param datadir Path to the data directory.
 * @param opts Creation and configuration options.
 * @returns A promise that resolves on success or rejects on error.
 */
export function loadWallet(
  datadir: string,
  opts: BarkCreateOpts
): Promise<void> {
  return NitroArkHybridObject.loadWallet(datadir, opts);
}

/**
 * Closes the currently loaded wallet, clearing its state from memory.
 * @returns A promise that resolves on success or rejects on error.
 */
export function closeWallet(): Promise<void> {
  return NitroArkHybridObject.closeWallet();
}

// --- Wallet Info ---

/**
 * Gets the onchain and offchain balances for the loaded wallet.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving to the BarkBalance object.
 */
export function getBalance(no_sync: boolean = false): Promise<BarkBalance> {
  return NitroArkHybridObject.getBalance(no_sync);
}

/**
 * Gets a fresh onchain address for the loaded wallet.
 * @returns A promise resolving to the Bitcoin address string.
 */
export function getOnchainAddress(): Promise<string> {
  return NitroArkHybridObject.getOnchainAddress();
}

/**
 * Gets the list of onchain UTXOs as a JSON string for the loaded wallet.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving to the JSON string of UTXOs.
 */
export function getOnchainUtxos(no_sync: boolean = false): Promise<string> {
  return NitroArkHybridObject.getOnchainUtxos(no_sync);
}

/**
 * Gets the wallet's VTXO public key (hex string).
 * @param index Optional index of the VTXO pubkey to retrieve.
 * @returns A promise resolving to the hex-encoded public key string.
 */
export function getVtxoPubkey(index?: number): Promise<string> {
  return NitroArkHybridObject.getVtxoPubkey(index);
}

/**
 * Gets the list of VTXOs as a JSON string for the loaded wallet.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving to the JSON string of VTXOs.
 */
export function getVtxos(no_sync: boolean = false): Promise<string> {
  return NitroArkHybridObject.getVtxos(no_sync);
}

// --- Onchain Operations ---

/**
 * Sends funds using the onchain wallet.
 * @param destination The destination Bitcoin address.
 * @param amountSat The amount to send in satoshis.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving to the transaction ID string.
 */
export function sendOnchain(
  destination: string,
  amountSat: number,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.sendOnchain(destination, amountSat, no_sync);
}

/**
 * Sends all funds from the onchain wallet to a destination address.
 * @param destination The destination Bitcoin address.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving to the transaction ID string.
 */
export function drainOnchain(
  destination: string,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.drainOnchain(destination, no_sync);
}

/**
 * Sends funds to multiple recipients using the onchain wallet.
 * @param outputs An array of objects containing destination address and amountSat.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving to the transaction ID string.
 */
export function sendManyOnchain(
  outputs: BarkSendManyOutput[],
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.sendManyOnchain(outputs, no_sync);
}

// --- Lightning Operations ---

/**
 * Creates a Bolt 11 invoice.
 * @param amountMsat The amount in millisatoshis for the invoice.
 * @returns A promise resolving to the Bolt 11 invoice string.
 */
export function bolt11Invoice(amountMsat: number): Promise<string> {
  return NitroArkHybridObject.bolt11Invoice(amountMsat);
}

/**
 * Claims a Bolt 11 payment.
 * @param bolt11 The Bolt 11 invoice string to claim.
 * @returns A promise that resolves on success or rejects on error.
 */
export function claimBolt11Payment(bolt11: string): Promise<void> {
  return NitroArkHybridObject.claimBolt11Payment(bolt11);
}

// --- Ark Operations ---

/**
 * Refreshes VTXOs based on specified criteria for the loaded wallet.
 * @param refreshOpts Options specifying which VTXOs to refresh.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function refreshVtxos(
  refreshOpts: BarkRefreshOpts,
  no_sync: boolean = false
): Promise<string> {
  if (!refreshOpts.mode_type) {
    return Promise.reject(
      new Error('refreshVtxos requires refreshOpts.mode_type')
    );
  }
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
  return NitroArkHybridObject.refreshVtxos(refreshOpts, no_sync);
}

/**
 * Boards a specific amount from the onchain wallet into Ark.
 * @param amountSat The amount in satoshis to board.
 * @param no_sync If true, skips synchronization with the onchain wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function boardAmount(
  amountSat: number,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.boardAmount(amountSat, no_sync);
}

/**
 * Boards all available funds from the onchain wallet into Ark.
 * @param no_sync If true, skips synchronization with the onchain wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function boardAll(no_sync: boolean = false): Promise<string> {
  return NitroArkHybridObject.boardAll(no_sync);
}

/**
 * Sends funds offchain using Ark VTXOs.
 * @param destination Ark address (VTXO pubkey) or onchain Bitcoin address.
 * @param amountSat The amount in satoshis to send.
 * @param comment Optional comment.
 * @param no_sync If true, skips synchronization with the wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function send(
  destination: string,
  amountSat: number | null,
  comment: string | null = null,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.send(destination, amountSat, comment, no_sync);
}

/**
 * Sends an onchain payment via an Ark round.
 * @param destination The destination Bitcoin address.
 * @param amountSat The amount in satoshis to send.
 * @param no_sync If true, skips synchronization with the wallet. Defaults to false.
 * @returns A promise resolving to a JSON status string.
 */
export function sendRoundOnchain(
  destination: string,
  amountSat: number,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.sendRoundOnchain(destination, amountSat, no_sync);
}

// --- Offboarding / Exiting ---

/**
 * Offboards specific VTXOs to an optional onchain address.
 * @param vtxoIds Array of VtxoId strings to offboard.
 * @param optionalAddress Optional destination Bitcoin address (null if sending to internal wallet).
 * @param no_sync If true, skips synchronization with the wallet. Defaults to false.
 * @returns A promise resolving to a JSON result string.
 */
export function offboardSpecific(
  vtxoIds: string[],
  optionalAddress: string | null = null,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.offboardSpecific(
    vtxoIds,
    optionalAddress,
    no_sync
  );
}

/**
 * Offboards all VTXOs to an optional onchain address.
 * @param optionalAddress Optional destination Bitcoin address (null if sending to internal wallet).
 * @param no_sync If true, skips synchronization with the wallet. Defaults to false.
 * @returns A promise resolving to a JSON result string.
 */
export function offboardAll(
  optionalAddress: string | null = null,
  no_sync: boolean = false
): Promise<string> {
  return NitroArkHybridObject.offboardAll(optionalAddress, no_sync);
}

/**
 * Starts the exit process for specific VTXOs.
 * @param vtxoIds Array of VtxoId strings to start exiting.
 * @returns A promise resolving to a JSON status string.
 */
export function exitStartSpecific(vtxoIds: string[]): Promise<string> {
  return NitroArkHybridObject.exitStartSpecific(vtxoIds);
}

/**
 * Starts the exit process for all VTXOs in the wallet.
 * @returns A promise resolving to a JSON status string.
 */
export function exitStartAll(): Promise<string> {
  return NitroArkHybridObject.exitStartAll();
}

/**
 * Progresses the exit process once and returns the current status.
 * @returns A promise resolving to a JSON status string.
 */
export function exitProgressOnce(): Promise<string> {
  return NitroArkHybridObject.exitProgressOnce();
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
