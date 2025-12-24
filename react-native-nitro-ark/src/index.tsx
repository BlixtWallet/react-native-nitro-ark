import { NitroModules } from 'react-native-nitro-modules';
import type {
  NitroArk,
  BarkCreateOpts,
  BarkArkInfo,
  Bolt11Invoice,
  BarkSendManyOutput,
  ArkoorPaymentResult,
  LightningSendResult,
  OnchainPaymentResult,
  OffchainBalanceResult,
  OnchainBalanceResult,
  NewAddressResult,
  KeyPairResult,
  LightningReceive,
  BarkMovement as NitroBarkMovement,
  BarkMovementDestination as NitroBarkMovementDestination,
  BoardResult,
  RoundStatus,
} from './NitroArk.nitro';

export type BarkVtxo = {
  amount: number; // u64
  expiry_height: number; // u32
  server_pubkey: string;
  exit_delta: number; // u16
  anchor_point: string;
  point: string;
  state: 'Spendable' | 'Spent' | 'Locked' | 'unknown';
};

export type MovementStatus = 'pending' | 'finished' | 'failed' | 'cancelled';

export type BarkMovementDestination = NitroBarkMovementDestination & {
  payment_method:
    | 'ark'
    | 'bitcoin'
    | 'output-script'
    | 'invoice'
    | 'offer'
    | 'lightning-address'
    | 'custom';
};

export type BarkMovement = NitroBarkMovement & {
  status: MovementStatus;
  sent_to: BarkMovementDestination[];
  received_on: BarkMovementDestination[];
};

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
 * @param opts The options for wallet creation.
 * @returns A promise that resolves on success or rejects on error.
 */
export function createWallet(
  datadir: string,
  opts: BarkCreateOpts
): Promise<void> {
  return NitroArkHybridObject.createWallet(datadir, opts);
}

/**
 * Loads an existing wallet or creates a new one at the specified directory.
 * Once loaded, the wallet state is managed internally.
 * @param datadir Path to the data directory.
 * @param config The configuration options for the wallet.
 * @returns A promise that resolves on success or rejects on error.
 */
export function loadWallet(
  datadir: string,
  config: BarkCreateOpts
): Promise<void> {
  return NitroArkHybridObject.loadWallet(datadir, config);
}

/**
 * Closes the currently loaded wallet, clearing its state from memory.
 * @returns A promise that resolves on success or rejects on error.
 */
export function closeWallet(): Promise<void> {
  return NitroArkHybridObject.closeWallet();
}

/**
 * Refreshes the server state.
 * @returns A promise that resolves on success or rejects on error.
 */
export function refreshServer(): Promise<void> {
  return NitroArkHybridObject.refreshServer();
}

/**
 * Checks if a wallet is currently loaded.
 * @returns A promise resolving to true if a wallet is loaded, false otherwise.
 */
export function isWalletLoaded(): Promise<boolean> {
  return NitroArkHybridObject.isWalletLoaded();
}

/**
 * Registers all confirmed boards with the server.
 * @returns A promise that resolves on success.
 */
export function syncPendingBoards(): Promise<void> {
  return NitroArkHybridObject.syncPendingBoards();
}

/**
 * Runs wallet maintenance tasks for offchain.
 * This includes refreshing vtxos that need to be refreshed.
 * @returns A promise that resolves on success.
 */
export function maintenance(): Promise<void> {
  return NitroArkHybridObject.maintenance();
}

/**
 * Runs wallet maintenance tasks for both offchain and onchain.
 * This includes refreshing vtxos that need to be refreshed.
 * @returns A promise that resolves on success.
 */
export function maintenanceWithOnchain(): Promise<void> {
  return NitroArkHybridObject.maintenanceWithOnchain();
}

/**
 * Refreshes vtxos that need to be refreshed.
 * @returns A promise that resolves on success.
 */
export function maintenanceRefresh(): Promise<void> {
  return NitroArkHybridObject.maintenanceRefresh();
}

/**
 * Synchronizes the wallet with the blockchain.
 * @returns A promise that resolves on success.
 */
export function sync(): Promise<void> {
  return NitroArkHybridObject.sync();
}

/**
 * Synchronizes the Ark-specific exits.
 * @returns A promise that resolves on success.
 */
export function syncExits(): Promise<void> {
  return NitroArkHybridObject.syncExits();
}

/**
 * Synchronizes pending rounds.
 * @returns A promise that resolves on success.
 */
export function syncPendingRounds(): Promise<void> {
  return NitroArkHybridObject.syncPendingRounds();
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
 * Gets the offchain balance for the loaded wallet.
 * @returns A promise resolving to the OffchainBalanceResult object.
 */
export function offchainBalance(): Promise<OffchainBalanceResult> {
  return NitroArkHybridObject.offchainBalance();
}

/**
 * Derives the next keypair for the store.
 * @returns A promise resolving to the KeyPairResult object.
 */
export function deriveStoreNextKeypair(): Promise<KeyPairResult> {
  return NitroArkHybridObject.deriveStoreNextKeypair();
}

/**
 * Gets the wallet's VTXO public key (hex string).
 * @param index Index of the VTXO pubkey to retrieve.
 * @returns A promise resolving to the KeyPairResult object.
 */
export function peakKeyPair(index: number): Promise<KeyPairResult> {
  return NitroArkHybridObject.peakKeyPair(index);
}

/**
 * Peeks a derived address without advancing the wallet's address index.
 * @param index Index of the address to preview.
 * @returns A promise resolving to the NewAddressResult object.
 */
export function peakAddress(index: number): Promise<NewAddressResult> {
  return NitroArkHybridObject.peakAddress(index);
}

/**
 * Gets the wallet's Address.
 * @returns A promise resolving to NewAddressResult object.
 */
export function newAddress(): Promise<NewAddressResult> {
  return NitroArkHybridObject.newAddress();
}

/**
 * Signs a message with the private key at the specified index.
 * @param message The message to sign.
 * @param index The index of the keypair to use for signing.
 * @returns A promise resolving to the signature string.
 */
export function signMessage(message: string, index: number): Promise<string> {
  return NitroArkHybridObject.signMessage(message, index);
}

/**
 * Signs a message with the private key at the specified index.
 * @param message The message to sign.
 * @param mnemonic The BIP39 mnemonic phrase to use for signing.
 * @param network The network to use for signing.
 * @param index The index of the keypair to use for signing.
 * @returns A promise resolving to the signature string.
 */
export function signMesssageWithMnemonic(
  message: string,
  mnemonic: string,
  network: string,
  index: number
): Promise<string> {
  return NitroArkHybridObject.signMesssageWithMnemonic(
    message,
    mnemonic,
    network,
    index
  );
}

/**
 * Derives a keypair from a mnemonic.
 * @param mnemonic The mnemonic to derive the keypair from.
 * @param network The network to derive the keypair for.
 * @param index The index to derive the keypair from.
 * @returns A promise resolving to the KeyPairResult object.
 */

export function deriveKeypairFromMnemonic(
  mnemonic: string,
  network: string,
  index: number
): Promise<KeyPairResult> {
  return NitroArkHybridObject.deriveKeypairFromMnemonic(
    mnemonic,
    network,
    index
  );
}

/**
 * Verifies a signed message.
 * @param message The original message.
 * @param signature The signature to verify.
 * @param publicKey The public key corresponding to the private key used for signing.
 * @returns A promise resolving to true if the signature is valid, false otherwise.
 */
export function verifyMessage(
  message: string,
  signature: string,
  publicKey: string
): Promise<boolean> {
  return NitroArkHybridObject.verifyMessage(message, signature, publicKey);
}

/**
 * Gets a paginated list of wallet history (balance changes).
 * @returns A promise resolving to an array of BarkMovement objects.
 */
export function history(): Promise<BarkMovement[]> {
  return NitroArkHybridObject.history() as Promise<BarkMovement[]>;
}

/**
 * Gets the list of VTXOs as a JSON string for the loaded wallet.
 * @param no_sync If true, skips synchronization with the blockchain. Defaults to false.
 * @returns A promise resolving BarkVtxo[] array.
 */
export function vtxos(): Promise<BarkVtxo[]> {
  return NitroArkHybridObject.vtxos() as Promise<BarkVtxo[]>;
}

/**
 * Gets the first expiring VTXO blockheight for the loaded wallet.
 * @returns A promise resolving to the first expiring VTXO blockheight.
 */
export function getFirstExpiringVtxoBlockheight(): Promise<number | undefined> {
  return NitroArkHybridObject.getFirstExpiringVtxoBlockheight();
}

/**
 * Gets the next required refresh blockheight for the loaded wallet for the first expiring VTXO.
 * @returns A promise resolving to the next required refresh blockheight.
 */
export function getNextRequiredRefreshBlockheight(): Promise<
  number | undefined
> {
  return NitroArkHybridObject.getNextRequiredRefreshBlockheight();
}

/**
 * Gets the list of expiring VTXOs as a JSON Object of type BarkVtxo.
 * @param threshold The block height threshold to check for expiring VTXOs.
 * @returns A promise resolving BarkVtxo[] array.
 */

export function getExpiringVtxos(threshold: number): Promise<BarkVtxo[]> {
  return NitroArkHybridObject.getExpiringVtxos(threshold) as Promise<
    BarkVtxo[]
  >;
}

// --- Onchain Operations ---

/**
 * Gets the onchain balance for the loaded wallet.
 * @returns A promise resolving to the OnchainBalanceResult object.
 */
export function onchainBalance(): Promise<OnchainBalanceResult> {
  return NitroArkHybridObject.onchainBalance();
}

/**
 * Synchronizes the onchain state of the wallet.
 * @returns A promise that resolves on success.
 */
export function onchainSync(): Promise<void> {
  return NitroArkHybridObject.onchainSync();
}

/**
 * Gets the list of unspent onchain outputs as a JSON Object of type BarkVtxo.
 * @returns A promise resolving to the JSON string of unspent outputs.
 */
export function onchainListUnspent(): Promise<string> {
  return NitroArkHybridObject.onchainListUnspent();
}

/**
 * Gets the list of onchain UTXOs as a JSON string for the loaded wallet.
 * @returns A promise resolving to the JSON string of UTXOs.
 */
export function onchainUtxos(): Promise<string> {
  return NitroArkHybridObject.onchainUtxos();
}

/**
 * Gets a fresh onchain address for the loaded wallet.
 * @returns A promise resolving to the Bitcoin address string.
 */
export function onchainAddress(): Promise<string> {
  return NitroArkHybridObject.onchainAddress();
}

/**
 * Sends funds using the onchain wallet.
 * @param destination The destination Bitcoin address.
 * @param amountSat The amount to send in satoshis.
 * @returns A promise resolving to the OnchainPaymentResult object
 */
export function onchainSend(
  destination: string,
  amountSat: number
): Promise<OnchainPaymentResult> {
  return NitroArkHybridObject.onchainSend(destination, amountSat);
}

/**
 * Sends all funds from the onchain wallet to a destination address.
 * @param destination The destination Bitcoin address.
 * @returns A promise resolving to the transaction ID string.
 */
export function onchainDrain(destination: string): Promise<string> {
  return NitroArkHybridObject.onchainDrain(destination);
}

/**
 * Sends funds to multiple recipients using the onchain wallet.
 * @param outputs An array of objects containing destination address and amountSat.
 * @returns A promise resolving to the transaction ID string.
 */
export function onchainSendMany(
  outputs: BarkSendManyOutput[]
): Promise<string> {
  return NitroArkHybridObject.onchainSendMany(outputs);
}

// --- Lightning Operations ---

/**
 * Creates a Bolt 11 invoice.
 * @param amountMsat The amount in millisatoshis for the invoice.
 * @returns A promise resolving to Bolt11Invoice object.
 */
export function bolt11Invoice(amountMsat: number): Promise<Bolt11Invoice> {
  return NitroArkHybridObject.bolt11Invoice(amountMsat);
}

/**
 * Gets the status of a Lightning receive.
 * @param paymentHash The payment hash of the Lightning receive.
 * @returns A promise resolving to the Lightning receive status.
 */
export function lightningReceiveStatus(
  paymentHash: string
): Promise<LightningReceive | undefined> {
  return NitroArkHybridObject.lightningReceiveStatus(paymentHash);
}

/**
 * Checks if a Lightning payment has been received and returns the preimage if available.
 * @param paymentHash The payment hash of the Lightning payment.
 * @param wait Whether to wait for the payment to be received.
 * @returns A promise resolving to the preimage string if payment received, or null if not.
 */
export function checkLightningPayment(
  paymentHash: string,
  wait: boolean
): Promise<string | null> {
  return NitroArkHybridObject.checkLightningPayment(paymentHash, wait);
}

/**
 * Attempts to claim a Lightning payment, optionally using a claim token.
 * @param paymentHash The payment hash of the Lightning payment.
 * @param wait Whether to wait for the claim to complete.
 * @param token Optional claim token used when no spendable VTXOs are owned.
 * @returns A promise that resolves on success or rejects on error.
 */
export function tryClaimLightningReceive(
  paymentHash: string,
  wait: boolean,
  token?: string
): Promise<void> {
  return NitroArkHybridObject.tryClaimLightningReceive(
    paymentHash,
    wait,
    token
  );
}

/**
 * Checks and claims all open Lightning receives.
 * @param wait Whether to wait for the claim to complete.
 * @returns A promise that resolves on success or rejects on error.
 */
export function tryClaimAllLightningReceives(wait: boolean): Promise<void> {
  return NitroArkHybridObject.tryClaimAllLightningReceives(wait);
}

/**
 * Pays a Bolt11 Lightning invoice.
 * @param destination The Lightning invoice.
 * @param amountSat The amount in satoshis to send. Use 0 for invoice amount.
 * @returns A promise resolving to a LightningSendResult object
 */
export function payLightningInvoice(
  destination: string,
  amountSat?: number
): Promise<LightningSendResult> {
  return NitroArkHybridObject.payLightningInvoice(destination, amountSat);
}

/**
 * Sends a payment to a Bolt12 offer.
 * @param offer The Bolt12 offer.
 * @param amountSat The amount in satoshis to send. Use 0 for invoice amount.
 * @returns A promise resolving to a LightningSendResult object
 */
export function payLightningOffer(
  offer: string,
  amountSat?: number
): Promise<LightningSendResult> {
  return NitroArkHybridObject.payLightningOffer(offer, amountSat);
}

/**
 * Sends a payment to a Lightning Address.
 * @param addr The Lightning Address.
 * @param amountSat The amount in satoshis to send.
 * @param comment An optional comment.
 * @returns A promise resolving to a LightningSendResult object
 */
export function payLightningAddress(
  addr: string,
  amountSat: number,
  comment: string
): Promise<LightningSendResult> {
  return NitroArkHybridObject.payLightningAddress(addr, amountSat, comment);
}

// --- Ark Operations ---

/**
 * Boards a specific amount from the onchain wallet into Ark.
 * @param amountSat The amount in satoshis to board.
 * @returns A promise resolving to a BoardResult object
 */
export function boardAmount(amountSat: number): Promise<BoardResult> {
  return NitroArkHybridObject.boardAmount(amountSat);
}

/**
 * Boards all available funds from the onchain wallet into Ark.
 * @returns A promise resolving to a BoardResult object.
 */
export function boardAll(): Promise<BoardResult> {
  return NitroArkHybridObject.boardAll();
}

/**
 * Validates an Arkoor address.
 * @param address The Arkoor address to validate.
 * @returns A promise resolving to void.
 */
export function validateArkoorAddress(address: string): Promise<void> {
  return NitroArkHybridObject.validateArkoorAddress(address);
}

/**
 * Sends an Arkoor payment.
 * @param destination The destination Arkoor address.
 * @param amountSat The amount in satoshis to send.
 * @returns A promise resolving to the ArkoorPaymentResult object
 */
export function sendArkoorPayment(
  destination: string,
  amountSat: number
): Promise<ArkoorPaymentResult> {
  return NitroArkHybridObject.sendArkoorPayment(destination, amountSat);
}

/**
 * Sends an onchain payment via an Ark round.
 * @param destination The destination Bitcoin address.
 * @param amountSat The amount in satoshis to send.
 * @returns A promise resolving to the round status.
 */
export function sendRoundOnchainPayment(
  destination: string,
  amountSat: number
): Promise<RoundStatus> {
  return NitroArkHybridObject.sendRoundOnchainPayment(destination, amountSat);
}

// --- Offboarding / Exiting ---

/**
 * Offboards specific VTXOs to a destination address.
 * @param vtxoIds Array of VtxoId strings to offboard.
 * @param destinationAddress Destination Bitcoin address (if empty, sends to internal wallet).
 * @returns A promise resolving to the round status.
 */
export function offboardSpecific(
  vtxoIds: string[],
  destinationAddress: string
): Promise<RoundStatus> {
  return NitroArkHybridObject.offboardSpecific(vtxoIds, destinationAddress);
}

/**
 * Offboards all VTXOs to a destination address.
 * @param destinationAddress Destination Bitcoin address (if empty, sends to internal wallet).
 * @returns A promise resolving to the round status.
 */
export function offboardAll(destinationAddress: string): Promise<RoundStatus> {
  return NitroArkHybridObject.offboardAll(destinationAddress);
}

// --- Re-export types and enums ---
export type {
  NitroArk,
  BarkCreateOpts,
  BarkConfigOpts,
  BarkArkInfo,
  Bolt11Invoice,
  BoardResult,
  BarkSendManyOutput,
  ArkoorPaymentResult,
  LightningSendResult,
  OnchainPaymentResult,
  PaymentTypes,
  OffchainBalanceResult,
  OnchainBalanceResult,
  NewAddressResult,
  KeyPairResult,
  LightningReceive,
  LightningReceiveBalance,
  RoundStatus,
} from './NitroArk.nitro';
