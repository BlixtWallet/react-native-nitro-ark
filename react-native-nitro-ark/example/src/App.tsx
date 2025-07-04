import { useState, useEffect, useCallback } from 'react';
import {
  Text,
  View,
  StyleSheet,
  Button,
  ScrollView,
  Platform,
  SafeAreaView,
  TextInput,
  ActivityIndicator,
} from 'react-native';
import {
  DocumentDirectoryPath,
  exists,
  mkdir,
  unlink,
} from '@dr.pogodin/react-native-fs';
import * as NitroArk from 'react-native-nitro-ark';
import type {
  BarkBalance,
  BarkRefreshModeType,
  BarkSendManyOutput,
} from 'react-native-nitro-ark';

import AsyncStorage from '@react-native-async-storage/async-storage';

// Constants
const ARK_DATA_PATH = `${DocumentDirectoryPath}/bark_data`;
const MNEMONIC_STORAGE_KEY = 'NITRO_ARK_MNEMONIC';

// Helper to format satoshis
const formatSats = (sats: number): string => {
  if (isNaN(sats) || sats === undefined || sats === null) {
    return 'N/A sats';
  }
  return `${sats.toLocaleString()} sats (${(sats / 100000000).toFixed(8)} BTC)`;
};

export default function ArkApp() {
  const [mnemonic, setMnemonic] = useState<string | undefined>(undefined);
  const [balanceState, setBalanceState] = useState<
    Partial<BarkBalance> & { error?: string }
  >({});
  const [results, setResults] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // Input States
  const [destinationAddress, setDestinationAddress] = useState('');
  const [amountSat, setAmountSat] = useState('');
  const [comment, setComment] = useState('');
  const [vtxoIdsInput, setVtxoIdsInput] = useState(''); // Comma separated
  const [optionalAddress, setOptionalAddress] = useState('');
  const [invoiceAmount, setInvoiceAmount] = useState('1000');
  const [invoiceToClaim, setInvoiceToClaim] = useState('');

  // Ensure data directory exists on mount
  useEffect(() => {
    const setupDirectory = async () => {
      try {
        const dirExists = await exists(ARK_DATA_PATH);
        if (!dirExists) {
          await mkdir(ARK_DATA_PATH, {
            NSURLIsExcludedFromBackupKey: true, // iOS specific
          });
          console.log('Data directory created:', ARK_DATA_PATH);
        } else {
          console.log('Data directory exists:', ARK_DATA_PATH);
        }
      } catch (err: any) {
        console.error('Error setting up data directory:', err);
        setError(`Failed to setup data directory: ${err.message}`);
      }
    };
    setupDirectory();
  }, []);

  useEffect(() => {
    const loadSavedMnemonic = async () => {
      try {
        const savedMnemonic = await AsyncStorage.getItem(MNEMONIC_STORAGE_KEY);
        if (savedMnemonic) {
          console.log('Loaded saved mnemonic');
          setMnemonic(savedMnemonic);
        }
      } catch (err) {
        console.error('Error loading saved mnemonic:', err);
      }
    };

    loadSavedMnemonic();
  }, []);

  // Generic function runner to handle loading, results, and errors
  const runOperation = useCallback(
    async (
      operationName: string,
      operationFn: () => Promise<any>,
      updateStateFn?: (result: any) => void
    ) => {
      setIsLoading(true);
      setResults('');
      setError('');
      console.log(`Running operation: ${operationName}...`);
      try {
        const result = await operationFn();
        console.log(`${operationName} success:`, result);

        if (updateStateFn) {
          updateStateFn(result);
        } else {
          // Default: Display result as string (or JSON string)
          setResults(
            typeof result === 'object' || typeof result === 'undefined'
              ? (JSON.stringify(result, null, 2) ??
                  'Operation successful (no return value)')
              : String(result)
          );
        }
      } catch (err: any) {
        console.error(`${operationName} error:`, err);
        setError(err.message || 'An unknown error occurred');
      } finally {
        setIsLoading(false);
      }
    },
    [] // No dependencies, captures initial state setters
  );

  // --- Operation Handlers ---

  const handleCreateMnemonic = () => {
    runOperation(
      'createMnemonic',
      () => NitroArk.createMnemonic(),
      async (newMnemonic) => {
        setMnemonic(newMnemonic);
        // Save the new mnemonic
        try {
          await AsyncStorage.setItem(MNEMONIC_STORAGE_KEY, newMnemonic);
          console.log('New mnemonic saved successfully');
        } catch (err: any) {
          console.error('Error saving new mnemonic:', err);
          setError(
            'Failed to save mnemonic: ' + (err.message || 'Unknown error')
          );
        }
      }
    );
  };

  const handleClearMnemonic = async () => {
    setIsLoading(true);
    try {
      await AsyncStorage.removeItem(MNEMONIC_STORAGE_KEY);
      setMnemonic(undefined);
      setResults('Mnemonic cleared successfully');
    } catch (err: any) {
      setError('Failed to clear mnemonic: ' + (err.message || 'Unknown error'));
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateWallet = async () => {
    if (!mnemonic) {
      setError('Mnemonic is required to create a wallet.');
      return;
    }

    try {
      await unlink(ARK_DATA_PATH); // Clear existing data directory if it exists
    } catch (err: any) {
      console.error('Error clearing existing data directory:', err);
    }

    // const opts: NitroArk.BarkCreateOpts = {
    //   mnemonic: mnemonic,
    //   regtest: true,
    //   signet: false,
    //   bitcoin: false,
    //   config: {
    //     bitcoind: 'http://192.168.4.252:18443',
    //     asp: 'http://192.168.4.252:3535',
    //     bitcoind_user: 'polaruser',
    //     bitcoind_pass: 'polarpass',
    //     vtxo_refresh_expiry_threshold: 288,
    //     fallback_fee_rate: 100000,
    //   },
    // };

    const opts: NitroArk.BarkCreateOpts = {
      mnemonic: mnemonic,
      regtest: false,
      signet: true,
      bitcoin: false,
      config: {
        esplora: 'esplora.signet.2nd.dev',
        asp: 'ark.signet.2nd.dev',
        vtxo_refresh_expiry_threshold: 288,
        fallback_fee_rate: 100000,
      },
    };

    runOperation(
      'loadWallet',
      () => NitroArk.loadWallet(ARK_DATA_PATH, opts),
      () => {
        setResults('Wallet created successfully!');
      }
    );
  };

  const handleCloseWallet = () => {
    runOperation('closeWallet', () => NitroArk.closeWallet());
  };

  const handleIsWalletLoaded = () => {
    runOperation('isWalletLoaded', () => NitroArk.isWalletLoaded());
  };

  const handleGetBalance = (noSync: boolean) => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    const opName = `getBalance (noSync: ${noSync})`;
    runOperation(
      opName,
      () => NitroArk.getBalance(noSync),
      (balance: BarkBalance) => {
        setBalanceState({
          onchain: balance.onchain,
          offchain: balance.offchain,
          pending_exit: balance.pending_exit, // Ensure key matches TS definition
          error: undefined,
        });
        setResults(''); // Clear generic results as balance has its own display
      }
    );
  };

  const handleGetOnchainAddress = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation('getOnchainAddress', () => NitroArk.getOnchainAddress());
  };

  const handleGetOnchainUtxos = (noSync: boolean) => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation(`getOnchainUtxos (noSync: ${noSync})`, () =>
      NitroArk.getOnchainUtxos(noSync)
    );
  };

  const handleGetVtxoPubkey = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation('getVtxoPubkey', () => NitroArk.getVtxoPubkey());
  };

  const handleGetVtxos = (noSync: boolean) => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation(`getVtxos (noSync: ${noSync})`, () =>
      NitroArk.getVtxos(noSync)
    );
  };

  const handleSendOnchain = (noSync: boolean) => {
    if (!mnemonic || !destinationAddress || !amountSat) {
      setError('Mnemonic, Destination Address, and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation(`sendOnchain (noSync: ${noSync})`, () =>
      NitroArk.sendOnchain(destinationAddress, amountNum, noSync)
    );
  };

  const handleDrainOnchain = (noSync: boolean) => {
    if (!mnemonic || !destinationAddress) {
      setError('Mnemonic and Destination Address are required.');
      return;
    }
    runOperation(`drainOnchain (noSync: ${noSync})`, () =>
      NitroArk.drainOnchain(destinationAddress, noSync)
    );
  };

  const handleSendManyOnchain = (noSync: boolean) => {
    if (!mnemonic || !destinationAddress || !amountSat) {
      setError(
        'Mnemonic, at least one Destination Address, and corresponding Amount are required.'
      );
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified for the first output.');
      return;
    }
    // Example: Using inputs for a single output in sendMany
    const outputs: BarkSendManyOutput[] = [
      { destination: destinationAddress, amountSat: amountNum },
      // Add more outputs here if needed, maybe from a more complex input UI
    ];
    runOperation(`sendManyOnchain (noSync: ${noSync})`, () =>
      NitroArk.sendManyOnchain(outputs, noSync)
    );
  };

  const handleRefreshVtxos = (mode: BarkRefreshModeType, noSync: boolean) => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    let refreshOpts: NitroArk.BarkRefreshOpts = { mode_type: mode };

    if (mode === 'Specific') {
      const ids = vtxoIdsInput
        .split(',')
        .map((id) => id.trim())
        .filter((id) => id);
      if (ids.length === 0) {
        setError('Specific VTXO IDs are required for this refresh mode.');
        return;
      }
      refreshOpts.specific_vtxo_ids = ids;
    } else if (mode === 'ThresholdBlocks' || mode === 'ThresholdHours') {
      // Example threshold - ideally get from input
      refreshOpts.threshold_value = 10; // Example: 10 blocks/hours
    }

    runOperation(`refreshVtxos (mode: ${mode}, noSync: ${noSync})`, () =>
      NitroArk.refreshVtxos(refreshOpts, noSync)
    );
  };

  const handleBoardAmount = (noSync: boolean) => {
    if (!mnemonic || !amountSat) {
      setError('Mnemonic and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation(`boardAmount (noSync: ${noSync})`, () =>
      NitroArk.boardAmount(amountNum, noSync)
    );
  };

  const handleBoardAll = (noSync: boolean) => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation(`boardAll (noSync: ${noSync})`, () =>
      NitroArk.boardAll(noSync)
    );
  };

  const handleSendArk = (noSync: boolean) => {
    if (!mnemonic || !destinationAddress || !amountSat) {
      setError('Mnemonic, Destination, and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    // Use comment from state, pass null if empty
    const commentToSend = comment.trim() === '' ? null : comment.trim();
    runOperation(`send (Ark) (noSync: ${noSync})`, () =>
      NitroArk.send(destinationAddress, amountNum, commentToSend, noSync)
    );
  };

  const handleSendRoundOnchain = (noSync: boolean) => {
    if (!mnemonic || !destinationAddress || !amountSat) {
      setError('Mnemonic, Destination Address, and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation(`sendRoundOnchain (noSync: ${noSync})`, () =>
      NitroArk.sendRoundOnchain(destinationAddress, amountNum, noSync)
    );
  };

  const handleOffboardSpecific = (noSync: boolean) => {
    if (!mnemonic || !vtxoIdsInput) {
      setError('Mnemonic and VTXO IDs are required.');
      return;
    }
    const ids = vtxoIdsInput
      .split(',')
      .map((id) => id.trim())
      .filter((id) => id);
    if (ids.length === 0) {
      setError('At least one VTXO ID is required.');
      return;
    }
    const addrToSend =
      optionalAddress.trim() === '' ? null : optionalAddress.trim();
    runOperation(`offboardSpecific (noSync: ${noSync})`, () =>
      NitroArk.offboardSpecific(ids, addrToSend, noSync)
    );
  };

  const handleOffboardAll = (noSync: boolean) => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    const addrToSend =
      optionalAddress.trim() === '' ? null : optionalAddress.trim();
    runOperation(`offboardAll (noSync: ${noSync})`, () =>
      NitroArk.offboardAll(addrToSend, noSync)
    );
  };

  const handleExitStartSpecific = (noSync: boolean) => {
    if (!mnemonic || !vtxoIdsInput) {
      setError('Mnemonic and VTXO IDs are required.');
      return;
    }
    const ids = vtxoIdsInput
      .split(',')
      .map((id) => id.trim())
      .filter((id) => id);
    if (ids.length === 0) {
      setError('At least one VTXO ID is required.');
      return;
    }
    runOperation(`exitStartSpecific (noSync: ${noSync})`, () =>
      NitroArk.exitStartSpecific(ids)
    );
  };

  const handleExitStartAll = (noSync: boolean) => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation(`exitStartAll (noSync: ${noSync})`, () =>
      NitroArk.exitStartAll()
    );
  };

  const handleExitProgressOnce = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation('exitProgressOnce', () => NitroArk.exitProgressOnce());
  };

  const handleCreateInvoice = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    const amount = parseInt(invoiceAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation(
      'bolt11Invoice',
      () => NitroArk.bolt11Invoice(amount),
      (invoice) => {
        setResults(`Created Invoice: ${invoice}`);
        setInvoiceToClaim(invoice);
      }
    );
  };

  const handleClaimPayment = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    if (!invoiceToClaim) {
      setError('Invoice to claim is required.');
      return;
    }
    runOperation(
      'claimBolt11Payment',
      () => NitroArk.claimBolt11Payment(invoiceToClaim),
      () => {
        setResults('Successfully claimed payment!');
      }
    );
  };

  // --- Render ---
  const canUseWallet = !!mnemonic;
  const walletOpsButtonDisabled = isLoading || !canUseWallet;

  return (
    <SafeAreaView style={styles.scrollContainer}>
      <ScrollView contentContainerStyle={styles.container}>
        <Text style={styles.headerText}>React Native Nitro Ark Test</Text>

        {/* --- Status & Mnemonic --- */}
        <Text style={styles.statusText}>Data Directory: {ARK_DATA_PATH}</Text>
        {mnemonic && (
          <View>
            <Text style={styles.statusText}>Mnemonic:</Text>
            <Text style={styles.mnemonicText} selectable={true}>
              {mnemonic}
            </Text>
          </View>
        )}

        {/* --- Management --- */}
        <Text style={styles.sectionHeader}>Management</Text>
        <View style={styles.buttonContainer}>
          <Button
            title="Generate Mnemonic"
            onPress={handleCreateMnemonic}
            disabled={isLoading || !!mnemonic} // Disable if already generated
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Clear Mnemonic"
            onPress={handleClearMnemonic}
            disabled={isLoading || !mnemonic} // Disable if no mnemonic
            color="#ff6666" // Red color to indicate destructive action
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Create Wallet"
            onPress={handleCreateWallet}
            disabled={isLoading || !mnemonic} // Disable if no mnemonic or already created
          />
        </View>

        <View style={styles.buttonContainer}>
          <Button
            title="Close Wallet"
            onPress={handleCloseWallet}
            disabled={isLoading || !mnemonic} // Disable if no mnemonic or already created
          />
        </View>

        <View style={styles.buttonContainer}>
          <Button
            title="Check Wallet Status"
            onPress={handleIsWalletLoaded}
            disabled={isLoading}
          />
        </View>

        {/* --- Wallet Info --- */}
        <Text style={styles.sectionHeader}>Wallet Info</Text>
        <View style={styles.balanceContainer}>
          <Text style={styles.balanceHeader}>Wallet Balance</Text>
          <Text style={styles.balanceText}>
            Onchain: {formatSats(balanceState.onchain ?? 0)}
          </Text>
          <Text style={styles.balanceText}>
            Offchain: {formatSats(balanceState.offchain ?? 0)}
          </Text>
          <Text style={styles.balanceText}>
            Pending Exit: {formatSats(balanceState.pending_exit ?? 0)}
          </Text>
          {balanceState.error && (
            <Text style={styles.errorText}>Error: {balanceState.error}</Text>
          )}
        </View>

        {results && (
          <View style={styles.resultContainer}>
            <Text style={styles.resultHeader}>Last Operation Result:</Text>
            <Text style={styles.resultText} selectable={true}>
              {results}
            </Text>
          </View>
        )}
        {error && (
          <View style={styles.errorContainer}>
            <Text style={styles.errorHeader}>Error:</Text>
            <Text style={styles.errorText} selectable={true}>
              {error}
            </Text>
          </View>
        )}
        <View style={styles.buttonContainer}>
          <Button
            title="Get Balance (Sync)"
            onPress={() => handleGetBalance(false)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get Balance (No Sync)"
            onPress={() => handleGetBalance(true)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get Onchain Address"
            onPress={handleGetOnchainAddress}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get Onchain UTXOs (Sync)"
            onPress={() => handleGetOnchainUtxos(false)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get Onchain UTXOs (No Sync)"
            onPress={() => handleGetOnchainUtxos(true)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get VTXO Pubkey"
            onPress={handleGetVtxoPubkey}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get VTXOs (Sync)"
            onPress={() => handleGetVtxos(false)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get VTXOs (No Sync)"
            onPress={() => handleGetVtxos(true)}
            disabled={walletOpsButtonDisabled}
          />
        </View>

        {/* --- Inputs for Operations --- */}
        <Text style={styles.sectionHeader}>Operation Inputs</Text>
        <View style={styles.inputContainer}>
          <Text style={styles.inputLabel}>Destination Address / Pubkey:</Text>
          <TextInput
            style={styles.input}
            value={destinationAddress}
            onChangeText={setDestinationAddress}
            placeholder="Enter Bitcoin Address or VTXO Pubkey"
            autoCapitalize="none"
            autoCorrect={false}
          />
        </View>
        <View style={styles.inputContainer}>
          <Text style={styles.inputLabel}>Amount (Satoshis):</Text>
          <TextInput
            style={styles.input}
            value={amountSat}
            onChangeText={setAmountSat}
            placeholder="e.g., 100000"
            keyboardType="numeric"
          />
        </View>
        <View style={styles.inputContainer}>
          <Text style={styles.inputLabel}>Comment (for Ark Send):</Text>
          <TextInput
            style={styles.input}
            value={comment}
            onChangeText={setComment}
            placeholder="Optional comment"
          />
        </View>
        <View style={styles.inputContainer}>
          <Text style={styles.inputLabel}>VTXO IDs (Comma-separated):</Text>
          <TextInput
            style={styles.input}
            value={vtxoIdsInput}
            onChangeText={setVtxoIdsInput}
            placeholder="vtxo_id_1,vtxo_id_2,..."
            autoCapitalize="none"
            autoCorrect={false}
          />
        </View>
        <View style={styles.inputContainer}>
          <Text style={styles.inputLabel}>Optional Address (Offboard):</Text>
          <TextInput
            style={styles.input}
            value={optionalAddress}
            onChangeText={setOptionalAddress}
            placeholder="Leave empty for internal address"
            autoCapitalize="none"
            autoCorrect={false}
          />
        </View>

        {/* --- Onchain Operations --- */}
        <Text style={styles.sectionHeader}>Onchain Operations</Text>
        <View style={styles.buttonContainer}>
          <Button
            title="Send Onchain"
            onPress={() => handleSendOnchain(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Drain Onchain"
            onPress={() => handleDrainOnchain(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Send Many Onchain (Uses Inputs for One)"
            onPress={() => handleSendManyOnchain(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>

        {/* --- Lightning Operations --- */}
        <Text style={styles.sectionHeader}>Lightning Operations</Text>
        <View style={styles.inputContainer}>
          <Text style={styles.inputLabel}>Invoice Amount (Satoshis):</Text>
          <TextInput
            style={styles.input}
            value={invoiceAmount}
            onChangeText={setInvoiceAmount}
            placeholder="e.g., 1000"
            keyboardType="numeric"
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Create Invoice"
            onPress={handleCreateInvoice}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.inputContainer}>
          <Text style={styles.inputLabel}>Invoice to Claim:</Text>
          <TextInput
            style={[styles.input, { height: 80 }]}
            value={invoiceToClaim}
            onChangeText={setInvoiceToClaim}
            placeholder="lnbc..."
            multiline
            selectTextOnFocus
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Claim Payment"
            onPress={handleClaimPayment}
            disabled={walletOpsButtonDisabled}
          />
        </View>

        {/* --- Ark Operations --- */}
        <Text style={styles.sectionHeader}>Ark Operations</Text>
        {/* Add buttons for different refresh modes */}
        <View style={styles.buttonContainer}>
          <Button
            title="Refresh VTXOs (Default)"
            onPress={() => handleRefreshVtxos('DefaultThreshold', false)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Refresh VTXOs (Specific - Use Input)"
            onPress={() => handleRefreshVtxos('Specific', false)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Refresh VTXOs (All)"
            onPress={() => handleRefreshVtxos('All', false)}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonSpacer} />
        <View style={styles.buttonContainer}>
          <Button
            title="Board Amount (Use Input)"
            onPress={() => handleBoardAmount(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Board All"
            onPress={() => handleBoardAll(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonSpacer} />
        <View style={styles.buttonContainer}>
          <Button
            title="Send (Ark - Use Inputs)"
            onPress={() => handleSendArk(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Send Round Onchain (Use Inputs)"
            onPress={() => handleSendRoundOnchain(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>

        {/* --- Offboarding / Exiting --- */}
        <Text style={styles.sectionHeader}>Offboarding / Exiting</Text>
        <View style={styles.buttonContainer}>
          <Button
            title="Offboard Specific (Use Inputs)"
            onPress={() => handleOffboardSpecific(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Offboard All (Use Optional Address)"
            onPress={() => handleOffboardAll(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonSpacer} />
        <View style={styles.buttonContainer}>
          <Button
            title="Exit Start Specific (Use VTXO ID Input)"
            onPress={() => handleExitStartSpecific(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Exit Start All"
            onPress={() => handleExitStartAll(false)} // Default to sync=false
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Exit Progress Once"
            onPress={handleExitProgressOnce}
            disabled={walletOpsButtonDisabled}
          />
        </View>

        {/* Spacer at the bottom */}
        <View style={{ height: 100 }} />
      </ScrollView>

      {/* Loading Indicator Overlay */}
      {isLoading && (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#ffffff" />
        </View>
      )}
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  scrollContainer: {
    flex: 1,
    backgroundColor: '#f0f0f0',
  },
  container: {
    padding: 15,
    paddingTop: Platform.OS === 'ios' ? 20 : 35,
  },
  headerText: {
    fontSize: 22,
    fontWeight: 'bold',
    marginBottom: 15,
    textAlign: 'center',
    color: '#333',
  },
  sectionHeader: {
    fontSize: 18,
    fontWeight: '600',
    marginTop: 20,
    marginBottom: 10,
    color: '#555',
    borderBottomWidth: 1,
    borderBottomColor: '#ddd',
    paddingBottom: 5,
  },
  statusText: {
    fontSize: 16,
    marginVertical: 5,
    textAlign: 'center',
    color: '#444',
  },
  mnemonicText: {
    fontSize: 14,
    marginVertical: 8,
    textAlign: 'center',
    color: 'blue',
    padding: 8,
    backgroundColor: '#e0e0ff',
    borderRadius: 4,
    fontFamily: Platform.OS === 'ios' ? 'Courier New' : 'monospace',
  },
  balanceContainer: {
    width: '100%',
    marginVertical: 15,
    padding: 15,
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 8,
    backgroundColor: '#fff',
  },
  balanceHeader: {
    fontSize: 16,
    fontWeight: 'bold',
    marginBottom: 8,
    textAlign: 'center',
  },
  balanceText: {
    fontSize: 14,
    marginVertical: 3,
  },
  inputContainer: {
    marginVertical: 10,
    width: '100%',
  },
  inputLabel: {
    fontSize: 14,
    fontWeight: '500',
    marginBottom: 4,
    color: '#333',
  },
  input: {
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 5,
    paddingHorizontal: 10,
    paddingVertical: 8,
    fontSize: 14,
    backgroundColor: '#fff',
    width: '100%',
  },
  buttonContainer: {
    marginVertical: 5,
  },
  buttonSpacer: {
    height: 10,
  },
  resultContainer: {
    marginTop: 15,
    padding: 10,
    backgroundColor: '#e8f4e8',
    borderRadius: 5,
    borderWidth: 1,
    borderColor: '#c8e4c8',
  },
  resultHeader: {
    fontWeight: 'bold',
    marginBottom: 5,
    color: '#387038',
  },
  resultText: {
    fontSize: 13,
    color: '#333',
    fontFamily: Platform.OS === 'ios' ? 'Courier New' : 'monospace',
  },
  errorContainer: {
    marginTop: 15,
    padding: 10,
    backgroundColor: '#fdecea',
    borderRadius: 5,
    borderWidth: 1,
    borderColor: '#f8c6a7',
  },
  errorHeader: {
    fontWeight: 'bold',
    marginBottom: 5,
    color: '#a94442',
  },
  errorText: {
    fontSize: 13,
    color: '#a94442',
  },
  loadingContainer: {
    position: 'absolute',
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(0, 0, 0, 0.3)',
    zIndex: 10,
  },
});
