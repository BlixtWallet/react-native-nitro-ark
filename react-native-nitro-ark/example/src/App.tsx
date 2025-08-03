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
  BarkArkInfo,
  BarkConfigOpts,
  BarkSendManyOutput,
  OnchainBalanceResult,
  OffchainBalanceResult,
} from 'react-native-nitro-ark';

import AsyncStorage from '@react-native-async-storage/async-storage';

// Constants
const ARK_DATA_PATH = `${DocumentDirectoryPath}/bark_data`;
const MNEMONIC_STORAGE_KEY = 'NITRO_ARK_MNEMONIC';

// Helper to format satoshis
const formatSats = (sats: number | undefined): string => {
  if (sats === undefined || isNaN(sats)) {
    return 'N/A';
  }
  return `${sats.toLocaleString()} sats`;
};

export default function ArkApp() {
  const [mnemonic, setMnemonic] = useState<string | undefined>(undefined);
  const [arkInfo, setArkInfo] = useState<BarkArkInfo | undefined>();
  const [onchainBalance, setOnchainBalance] = useState<
    OnchainBalanceResult | undefined
  >();
  const [offchainBalance, setOffchainBalance] = useState<
    OffchainBalanceResult | undefined
  >();
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

    const opts: NitroArk.BarkCreateOpts = {
      mnemonic: mnemonic,
      regtest: true,
      signet: false,
      bitcoin: false,
      config: {
        bitcoind: 'http://localhost:18443',
        asp: 'http://localhost:3535',
        bitcoind_user: 'second',
        bitcoind_pass: 'ark',
        vtxo_refresh_expiry_threshold: 288,
        fallback_fee_rate: 10000,
      },
    };

    // const opts: NitroArk.BarkCreateOpts = {
    //   mnemonic: mnemonic,
    //   regtest: false,
    //   signet: true,
    //   bitcoin: false,
    //   config: {
    //     esplora: 'esplora.signet.2nd.dev',
    //     asp: 'ark.signet.2nd.dev',
    //     vtxo_refresh_expiry_threshold: 288,
    //     fallback_fee_rate: 100000,
    //   },
    // };

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

  const handlePersistConfig = () => {
    // Example config, ideally from UI inputs
    const opts: BarkConfigOpts = {
      fallback_fee_rate: 1200,
    };
    runOperation('persistConfig', () => NitroArk.persistConfig(opts));
  };

  const handleMaintenance = () => {
    runOperation('maintenance', () => NitroArk.maintenance());
  };

  const handleSync = () => {
    runOperation('sync', () => NitroArk.sync());
  };

  const handleSyncExits = () => {
    runOperation('syncExits', () => NitroArk.syncExits());
  };

  const handleSyncRounds = () => {
    runOperation('syncRounds', () => NitroArk.syncRounds());
  };

  const handleGetArkInfo = () => {
    runOperation('getArkInfo', () => NitroArk.getArkInfo(), setArkInfo);
  };

  const handleGetOnchainBalance = () => {
    runOperation(
      'onchainBalance',
      () => NitroArk.onchainBalance(),
      (balance) => {
        setOnchainBalance(balance);
        setResults(
          `Onchain Balance: ${JSON.stringify(balance, null, 2)}`
        );
      }
    );
  };

  const handleGetOffchainBalance = () => {
    runOperation(
      'offchainBalance',
      () => NitroArk.offchainBalance(),
      (balance) => {
        setOffchainBalance(balance);
        setResults(
          `Offchain Balance: ${JSON.stringify(balance, null, 2)}`
        );
      }
    );
  };

  const handleGetOnchainAddress = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation('onchainAddress', () => NitroArk.onchainAddress());
  };

  const handleGetOnchainUtxos = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation('onchainUtxos', () => NitroArk.onchainUtxos());
  };

  const handleGetVtxos = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation('getVtxos', () => NitroArk.getVtxos());
  };

  const handleSendOnchain = () => {
    if (!destinationAddress || !amountSat) {
      setError('Mnemonic, Destination Address, and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation('onchainSend', () =>
      NitroArk.onchainSend(destinationAddress, amountNum)
    );
  };

  const handleDrainOnchain = () => {
    if (!mnemonic || !destinationAddress) {
      setError('Mnemonic and Destination Address are required.');
      return;
    }
    runOperation('onchainDrain', () =>
      NitroArk.onchainDrain(destinationAddress)
    );
  };

  const handleSendManyOnchain = () => {
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
    runOperation('onchainSendMany', () => NitroArk.onchainSendMany(outputs));
  };

  const handleBoardAmount = () => {
    if (!amountSat) {
      setError('Mnemonic and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation('boardAmount', () => NitroArk.boardAmount(amountNum));
  };

  const handleBoardAll = () => {
    if (!mnemonic) {
      setError('Mnemonic required');
      return;
    }
    runOperation('boardAll', () => NitroArk.boardAll());
  };

  const handleSendArkoorPayment = () => {
    if (!destinationAddress || !amountSat) {
      setError('Mnemonic, Destination, and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation('sendArkoorPayment', () =>
      NitroArk.sendArkoorPayment(destinationAddress, amountNum)
    );
  };

  const handleSendLightningPayment = () => {
    if (!destinationAddress) {
      setError('Mnemonic and Destination (invoice) are required.');
      return;
    }
    // Amount can be 0 to use invoice's amount
    const amountNum = parseInt(amountSat, 10) || 0;
    if (isNaN(amountNum) || amountNum < 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation('sendLightningPayment', () =>
      NitroArk.sendLightningPayment(destinationAddress, amountNum)
    );
  };

  const handleSendLnaddr = () => {
    if (!destinationAddress || !amountSat) {
      setError('Mnemonic, Destination (lnaddr), and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation('sendLnaddr', () =>
      NitroArk.sendLnaddr(destinationAddress, amountNum, comment)
    );
  };

  const handleSendRoundOnchainPayment = () => {
    if (!destinationAddress || !amountSat) {
      setError('Mnemonic, Destination Address, and Amount are required.');
      return;
    }
    const amountNum = parseInt(amountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Invalid amount specified.');
      return;
    }
    runOperation('sendRoundOnchainPayment', () =>
      NitroArk.sendRoundOnchainPayment(destinationAddress, amountNum)
    );
  };

  const handleOffboardSpecific = () => {
    if (!vtxoIdsInput || !optionalAddress) {
      setError('Mnemonic, VTXO IDs, and Destination Address are required.');
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
    runOperation('offboardSpecific', () =>
      NitroArk.offboardSpecific(ids, optionalAddress)
    );
  };

  const handleOffboardAll = () => {
    if (!optionalAddress) {
      setError('Mnemonic and Destination Address are required.');
      return;
    }
    runOperation('offboardAll', () => NitroArk.offboardAll(optionalAddress));
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
      'finishLightningReceive',
      () => NitroArk.finishLightningReceive(invoiceToClaim),
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
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Check Wallet Status"
            onPress={handleIsWalletLoaded}
            disabled={isLoading}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Persist Config"
            onPress={handlePersistConfig}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Maintenance"
            onPress={handleMaintenance}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Sync"
            onPress={handleSync}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Sync Exits"
            onPress={handleSyncExits}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Sync Rounds"
            onPress={handleSyncRounds}
            disabled={walletOpsButtonDisabled}
          />
        </View>

        {/* --- Wallet Info --- */}
        <Text style={styles.sectionHeader}>Wallet Info</Text>
        <View style={styles.balanceContainer}>
          <Text style={styles.balanceHeader}>Wallet Balance</Text>
          <Text style={styles.balanceText}>
            Onchain (Confirmed): {formatSats(onchainBalance?.confirmed)}
          </Text>
          <Text style={styles.balanceText}>
            Onchain (Immature): {formatSats(onchainBalance?.immature)}
          </Text>
          <Text style={styles.balanceText}>
            Onchain (Pending):{' '}
            {formatSats(
              (onchainBalance?.trusted_pending ?? 0) +
                (onchainBalance?.untrusted_pending ?? 0)
            )}
          </Text>
          <Text style={styles.balanceText}>
            Offchain (Spendable): {formatSats(offchainBalance?.spendable)}
          </Text>
          <Text style={styles.balanceText}>
            Offchain (Pending Send):{' '}
            {formatSats(offchainBalance?.pending_lightning_send)}
          </Text>
          <Text style={styles.balanceText}>
            Offchain (Pending Exit): {formatSats(offchainBalance?.pending_exit)}
          </Text>
        </View>

        {arkInfo && (
          <View style={styles.resultContainer}>
            <Text style={styles.resultHeader}>Ark Info:</Text>
            <Text style={styles.resultText} selectable={true}>
              {JSON.stringify(arkInfo, null, 2)}
            </Text>
          </View>
        )}

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
            title="Get Ark Info"
            onPress={handleGetArkInfo}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get Onchain Balance"
            onPress={handleGetOnchainBalance}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get Offchain Balance"
            onPress={handleGetOffchainBalance}
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
            title="Get Onchain UTXOs"
            onPress={() => handleGetOnchainUtxos()}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Get VTXOs"
            onPress={() => handleGetVtxos()}
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
            onPress={() => handleSendOnchain()}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Drain Onchain"
            onPress={() => handleDrainOnchain()}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Send Many Onchain"
            onPress={() => handleSendManyOnchain()}
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
        <Text style={styles.sectionHeader}>Ark & Lightning Payments</Text>
        <View style={styles.buttonContainer}>
          <Button
            title="Board Amount (Use Input)"
            onPress={handleBoardAmount}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Board All"
            onPress={handleBoardAll}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonSpacer} />
        <View style={styles.buttonContainer}>
          <Button
            title="Send Arkoor Payment"
            onPress={handleSendArkoorPayment}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Send Lightning Payment"
            onPress={handleSendLightningPayment}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Send to Lightning Address"
            onPress={handleSendLnaddr}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Send Round Onchain Payment"
            onPress={() => handleSendRoundOnchainPayment()}
            disabled={walletOpsButtonDisabled}
          />
        </View>

        {/* --- Offboarding / Exiting --- */}
        <Text style={styles.sectionHeader}>Offboarding / Exiting</Text>
        <View style={styles.buttonContainer}>
          <Button
            title="Offboard Specific"
            onPress={() => handleOffboardSpecific()}
            disabled={walletOpsButtonDisabled}
          />
        </View>
        <View style={styles.buttonContainer}>
          <Button
            title="Offboard All"
            onPress={() => handleOffboardAll()}
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
