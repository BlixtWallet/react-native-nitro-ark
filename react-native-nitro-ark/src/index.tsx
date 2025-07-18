import { NitroModules } from 'react-native-nitro-modules';
import type {
  NitroArk,
  BarkCreateOpts,
  BarkConfigOpts,
  BarkArkInfo,
  BarkSendManyOutput,
  ArkoorPaymentResult,
  Bolt11PaymentResult,
  LnurlPaymentResult,
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

/**
 * Checks if a wallet is currently loaded.
 * @returns A promise resolving to true if a wallet is loaded, false otherwise.
 */
export function isWalletLoaded(): Promise<boolean> {
  return NitroArkHybridObject.isWalletLoaded();
}

/**
 * Persists wallet configuration.
 * @param opts The configuration options to persist.
 * @returns A promise that resolves on success or rejects on error.
 */
export function persistConfig(opts: BarkConfigOpts): Promise<void> {
  return NitroArkHybridObject.persistConfig(opts);
}

/**
 * Runs wallet maintenance tasks.
 * @returns A promise that resolves on success.
 */
export function maintenance(): Promise<void> {
  return NitroArkHybridObject.maintenance();
}

/**
 * Synchronizes the wallet with the blockchain.
 * @returns A promise that resolves on success.
 */
export function sync(): Promise<void> {
  return NitroArkHybridObject.sync();
}

/**
 * Synchronizes the Ark-specific parts of the wallet.
 * @returns A promise that resolves on success.
 */
export function syncArk(): Promise<void> {
  return NitroArkHybridObject.syncArk();
}

/**
 * Synchronizes the rounds of the wallet.
 * @returns A promise that resolves on success.
 */
export function syncRounds(): Promise<void> {
  return NitroArkHybridObject.syncRounds();
}

// --- Wallet Info ---

/**
 * Gets the Ark-specific information.
 * @returns A promise resolving to the BarkArkInfo object.
 */
export function getArkInfo(): Promise<BarkArkInfo> {
  return NitroArkHybridObject.getArkInfo();
}

/**
 * Gets the onchain balance for the loaded wallet.
 * @returns A promise resolving to the onchain balance in satoshis.
 */
export function onchainBalance(): Promise<number> {
  return NitroArkHybridObject.onchainBalance();
}

/**
 * Gets the offchain balance for the loaded wallet.
 * @returns A promise resolving to the offchain balance in satoshis.
 */
export function offchainBalance(): Promise<number> {
  return NitroArkHybridObject.offchainBalance();
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
 * @param index Index of the VTXO pubkey to retrieve. Use u32::MAX for a new one.
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
 * Boards a specific amount from the onchain wallet into Ark.
 * @param amountSat The amount in satoshis to board.
 * @returns A promise resolving to a JSON status string.
 */
export function boardAmount(amountSat: number): Promise<string> {
  return NitroArkHybridObject.boardAmount(amountSat);
}

/**
 * Boards all available funds from the onchain wallet into Ark.
 * @returns A promise resolving to a JSON status string.
 */
export function boardAll(): Promise<string> {
  return NitroArkHybridObject.boardAll();
}

/**
 * Sends an Arkoor payment.
 * @param destination The destination Arkoor address.
 * @param amountSat The amount in satoshis to send.
 * @returns A promise resolving to a result string.
 */
export function sendArkoorPayment(
  destination: string,
  amountSat: number
): Promise<ArkoorPaymentResult> {
  return NitroArkHybridObject.sendArkoorPayment(destination, amountSat);
}

/**
 * Sends a Bolt11 payment.
 * @param destination The Bolt11 invoice.
 * @param amountSat The amount in satoshis to send. Use 0 for invoice amount.
 * @returns A promise resolving to a result string.
 */
export function sendBolt11Payment(
  destination: string,
  amountSat?: number
): Promise<Bolt11PaymentResult> {
  return NitroArkHybridObject.sendBolt11Payment(destination, amountSat);
}

/**
 * Sends a payment to a Lightning Address.
 * @param addr The Lightning Address.
 * @param amountSat The amount in satoshis to send.
 * @param comment An optional comment.
 * @returns A promise resolving to a result string.
 */
export function sendLnaddr(
  addr: string,
  amountSat: number,
  comment: string
): Promise<LnurlPaymentResult> {
  return NitroArkHybridObject.sendLnaddr(addr, amountSat, comment);
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
 * Offboards specific VTXOs to a destination address.
 * @param vtxoIds Array of VtxoId strings to offboard.
 * @param destinationAddress Destination Bitcoin address (if empty, sends to internal wallet).
 * @param no_sync If true, skips synchronization with the wallet. Defaults to false.
 * @returns A promise resolving to a JSON result string.
 */
export function offboardSpecific(
  vtxoIds: string[],
  destinationAddress: string
): Promise<string> {
  return NitroArkHybridObject.offboardSpecific(vtxoIds, destinationAddress);
}

/**
 * Offboards all VTXOs to a destination address.
 * @param destinationAddress Destination Bitcoin address (if empty, sends to internal wallet).
 * @param no_sync If true, skips synchronization with the wallet. Defaults to false.
 * @returns A promise resolving to a JSON result string.
 */
export function offboardAll(destinationAddress: string): Promise<string> {
  return NitroArkHybridObject.offboardAll(destinationAddress);
}

/**
 * Starts the exit process for specific VTXOs.
 * @param vtxoIds Array of VtxoId strings to start exiting.
 * @returns A promise resolving to a JSON status string.
 */
export function startExitForVtxos(vtxoIds: string[]): Promise<string> {
  return NitroArkHybridObject.exitStartSpecific(vtxoIds);
}

/**
 * Starts the exit process for all VTXOs in the wallet.
 * @returns A promise resolving to a JSON status string.
 */
export function startExitForEntireWallet(): Promise<string> {
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
  NitroArk,
  BarkCreateOpts,
  BarkConfigOpts,
  BarkArkInfo,
  BarkSendManyOutput,
  ArkoorPaymentResult,
  Bolt11PaymentResult,
  LnurlPaymentResult,
} from './NitroArk.nitro';
