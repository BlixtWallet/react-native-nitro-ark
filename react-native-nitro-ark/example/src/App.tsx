import { useState, useEffect, useCallback } from 'react';
import {
  Text,
  View,
  StyleSheet,
  ScrollView,
  NativeModules,
  Platform,
  SafeAreaView,
  TextInput,
  ActivityIndicator,
  TouchableOpacity,
} from 'react-native';
import RNFSTurbo from 'react-native-fs-turbo';

import * as NitroArk from 'react-native-nitro-ark';
import type {
  BarkArkInfo,
  BarkSendManyOutput,
  OnchainBalanceResult,
  OffchainBalanceResult,
} from 'react-native-nitro-ark';

import AsyncStorage from '@react-native-async-storage/async-storage';

// Constants
const ARK_DATA_PATH = `${RNFSTurbo.DocumentDirectoryPath}/bark_data`;
const MNEMONIC_STORAGE_KEY = 'NITRO_ARK_MNEMONIC';

const { NitroArkDemo } = NativeModules as {
  NitroArkDemo?: {
    loadWallet(
      datadir: string,
      mnemonic: string,
      config?: Record<string, any>
    ): Promise<void>;
    maintenanceRefresh(): Promise<void>;
    maintenance(): Promise<void>;
    tryClaimLightningReceive(
      paymentHash: string,
      wait: boolean,
      token?: string
    ): Promise<void>;
    offboardAll(destination: string): Promise<string>;
    peakKeyPair(index: number): Promise<string>;
    verifyMessage(
      message: string,
      signature: string,
      publicKey: string
    ): Promise<boolean>;
    bolt11Invoice(amountMsat: number): Promise<string>;
    isWalletLoaded(): Promise<boolean>;
    closeWallet(): Promise<void>;
    signMessage(message: string, index: number): Promise<string>;
    sync(): Promise<void>;
  };
};

// Helper to format satoshis
const formatSats = (sats: number | undefined): string => {
  if (sats === undefined || isNaN(sats)) {
    return 'N/A';
  }
  return `${sats.toLocaleString()} sats`;
};

const getWalletConfig = (mnemonic: string) => {
  const opts: NitroArk.BarkCreateOpts = {
    mnemonic: mnemonic,
    regtest: true,
    signet: false,
    bitcoin: false,
    config: {
      bitcoind:
        Platform.OS === 'android'
          ? 'http://192.168.4.72:18443'
          : 'http://localhost:18443',
      ark:
        Platform.OS === 'android'
          ? 'http://192.168.4.72:3535'
          : 'http://localhost:3535',
      bitcoind_user: 'second',
      bitcoind_pass: 'ark',
      vtxo_refresh_expiry_threshold: 48,
      fallback_fee_rate: 10000,
      htlc_recv_claim_delta: 18,
      vtxo_exit_margin: 12,
      round_tx_required_confirmations: 1,
    },
  };

  // const opts: NitroArk.BarkCreateOpts = {
  //   mnemonic: mnemonic,
  //   regtest: false,
  //   signet: true,
  //   bitcoin: false,
  //   config: {
  //     esplora: 'esplora.signet.2nd.dev',
  //     ark: 'ark.signet.2nd.dev',
  //     vtxo_refresh_expiry_threshold: 288,
  //     fallback_fee_rate: 100000,
  //   },
  // };

  return opts;
};

interface CustomButtonProps {
  title: string;
  onPress: () => void;
  disabled?: boolean;
  color?: string;
}

const CustomButton = ({
  title,
  onPress,
  disabled,
  color,
}: CustomButtonProps) => (
  <TouchableOpacity
    style={[
      styles.customButton,
      { backgroundColor: color || '#007AFF' },
      disabled && styles.customButtonDisabled,
    ]}
    onPress={onPress}
    disabled={disabled}
  >
    <Text style={styles.customButtonText}>{title}</Text>
  </TouchableOpacity>
);

export default function ArkApp() {
  const [mnemonic, setMnemonic] = useState<string | undefined>(undefined);
  const [arkInfo, setArkInfo] = useState<BarkArkInfo | undefined>();
  const [onchainBalance, setOnchainBalance] = useState<
    OnchainBalanceResult | undefined
  >();
  const [offchainBalance, setOffchainBalance] = useState<
    OffchainBalanceResult | undefined
  >();
  const [results, setResults] = useState<{ [key: string]: string }>({});
  const [error, setError] = useState<{ [key: string]: string }>({});
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // Input States
  const [onchainDestinationAddress, setOnchainDestinationAddress] =
    useState('');
  const [onchainAmountSat, setOnchainAmountSat] = useState('');
  const [arkDestinationAddress, setArkDestinationAddress] = useState('');
  const [arkAmountSat, setArkAmountSat] = useState('');
  const [arkComment, setArkComment] = useState('');
  const [vtxoIdsInput, setVtxoIdsInput] = useState(''); // Comma separated
  const [optionalAddress, setOptionalAddress] = useState('');
  const [invoiceAmount, setInvoiceAmount] = useState('1000');
  const [invoiceToClaim, setInvoiceToClaim] = useState('');
  const [messageToSign, setMessageToSign] = useState('hello world');
  const [signature, setSignature] = useState('');
  const [publicKeyForVerification, setPublicKeyForVerification] = useState('');
  const [arkoorAddressToValidate, setArkoorAddressToValidate] = useState('');
  const [paymentHash, setPaymentHash] = useState('');

  // Ensure data directory exists on mount
  useEffect(() => {
    const setupDirectory = async () => {
      try {
        const dirExists = RNFSTurbo.exists(ARK_DATA_PATH);
        if (!dirExists) {
          RNFSTurbo.mkdir(ARK_DATA_PATH, {
            NSURLIsExcludedFromBackupKey: true, // iOS specific
          });
          console.log('Data directory created:', ARK_DATA_PATH);
        } else {
          console.log('Data directory exists:', ARK_DATA_PATH);
        }
      } catch (err: any) {
        console.error('Error setting up data directory:', err);
        setError((prev) => ({
          ...prev,
          management: `Failed to setup data directory: ${err.message}`,
        }));
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
      section: string,
      updateStateFn?: (result: any) => void
    ) => {
      setIsLoading(true);
      setResults((prev) => ({ ...prev, [section]: '' }));
      setError((prev) => ({ ...prev, [section]: '' }));
      console.log(`Running operation: ${operationName}...`);
      try {
        const result = await operationFn();
        console.log(`${operationName} success:`, result);

        if (updateStateFn) {
          updateStateFn(result);
        } else {
          // Default: Display result as string (or JSON string)
          setResults((prev) => ({
            ...prev,
            [section]:
              typeof result === 'object' || typeof result === 'undefined'
                ? (JSON.stringify(result, null, 2) ??
                  'Operation successful (no return value)')
                : String(result),
          }));
        }
      } catch (err: any) {
        console.error(`${operationName} error:`, err);
        setError((prev) => ({
          ...prev,
          [section]: err.message || 'An unknown error occurred',
        }));
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
      'management',
      async (newMnemonic) => {
        setMnemonic(newMnemonic);
        // Save the new mnemonic
        try {
          await AsyncStorage.setItem(MNEMONIC_STORAGE_KEY, newMnemonic);
          console.log('New mnemonic saved successfully');
        } catch (err: any) {
          console.error('Error saving new mnemonic:', err);
          setError((prev) => ({
            ...prev,
            management:
              'Failed to save mnemonic: ' + (err.message || 'Unknown error'),
          }));
        }
      }
    );
  };

  const handleClearMnemonic = async () => {
    setIsLoading(true);
    try {
      await AsyncStorage.removeItem(MNEMONIC_STORAGE_KEY);
      RNFSTurbo.unlink(ARK_DATA_PATH); // Clear the data directory
      setMnemonic(undefined);
      setResults((prev) => ({
        ...prev,
        management: 'Mnemonic cleared successfully',
      }));
    } catch (err: any) {
      setError((prev) => ({
        ...prev,
        management:
          'Failed to clear mnemonic: ' + (err.message || 'Unknown error'),
      }));
    } finally {
      setIsLoading(false);
    }
  };

  const handleAndroidNativeLoadWallet = () => {
    if (Platform.OS !== 'android') {
      return;
    }
    if (!NitroArkDemo) {
      setError((prev) => ({
        ...prev,
        androidNative:
          'NitroArkDemo native module is not available (Android-only demo).',
      }));
      return;
    }
    if (!mnemonic) {
      setError((prev) => ({
        ...prev,
        androidNative: 'Mnemonic is required to load wallet.',
      }));
      return;
    }

    runOperation(
      'androidNativeLoadWallet',
      () =>
        NitroArkDemo.loadWallet(
          ARK_DATA_PATH,
          mnemonic,
          getWalletConfig(mnemonic)
        ),
      'androidNative'
    );
  };

  const handleAndroidNativeIsWalletLoaded = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    runOperation(
      'androidNativeIsWalletLoaded',
      () => NitroArkDemo.isWalletLoaded(),
      'androidNative'
    );
  };

  const handleAndroidNativeCloseWallet = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    runOperation(
      'androidNativeCloseWallet',
      () => NitroArkDemo.closeWallet(),
      'androidNative'
    );
  };

  const handleAndroidNativeMaintenance = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    runOperation(
      'androidNativeMaintenance',
      () => NitroArkDemo.maintenance(),
      'androidNative'
    );
  };

  const handleAndroidNativeMaintenanceRefresh = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    runOperation(
      'androidNative',
      () => NitroArkDemo.maintenanceRefresh(),
      'androidNative'
    );
  };

  const handleAndroidNativeTryClaimLightningReceive = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    if (!paymentHash) {
      setError((prev) => ({
        ...prev,
        androidNative: 'Payment hash is required.',
      }));
      return;
    }
    runOperation(
      'androidNative',
      () => NitroArkDemo.tryClaimLightningReceive(paymentHash, true, undefined),
      'androidNative'
    );
  };

  const handleAndroidNativeOffboardAll = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    if (!optionalAddress) {
      setError((prev) => ({
        ...prev,
        androidNative: 'Destination address is required.',
      }));
      return;
    }
    runOperation(
      'androidNative',
      () => NitroArkDemo.offboardAll(optionalAddress),
      'androidNative'
    );
  };

  const handleAndroidNativePeakKeyPair = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    runOperation(
      'androidNative',
      () => NitroArkDemo.peakKeyPair(0),
      'androidNative'
    );
  };

  const handleAndroidNativeVerifyMessage = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    if (!messageToSign || !signature || !publicKeyForVerification) {
      setError((prev) => ({
        ...prev,
        androidNative:
          'Message, signature, and public key are required for verification.',
      }));
      return;
    }
    runOperation(
      'androidNative',
      () =>
        NitroArkDemo.verifyMessage(
          messageToSign,
          signature,
          publicKeyForVerification
        ),
      'androidNative'
    );
  };

  const handleAndroidNativeBolt11Invoice = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    const amountMsat = parseInt(invoiceAmount || '0', 10) * 1000;
    runOperation(
      'androidNative',
      () => NitroArkDemo.bolt11Invoice(amountMsat),
      'androidNative'
    );
  };

  const handleAndroidNativeSignMessage = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    runOperation(
      'androidNative',
      () => NitroArkDemo.signMessage(messageToSign || '', 0),
      'androidNative'
    );
  };

  const handleAndroidNativeSync = () => {
    if (Platform.OS !== 'android' || !NitroArkDemo) {
      return;
    }
    runOperation('androidNative', () => NitroArkDemo.sync(), 'androidNative');
  };

  const handleCreateWallet = async () => {
    if (!mnemonic) {
      setError((prev) => ({
        ...prev,
        management: 'Mnemonic is required to create a wallet.',
      }));
      return;
    }

    runOperation(
      'createWallet',
      () => NitroArk.createWallet(ARK_DATA_PATH, getWalletConfig(mnemonic)),
      'management',
      () => {
        setResults((prev) => ({
          ...prev,
          management: 'Wallet created successfully!',
        }));
      }
    );
  };

  const handleLoadWallet = async () => {
    if (!mnemonic) {
      setError((prev) => ({
        ...prev,
        management: 'Mnemonic is required to load a wallet.',
      }));
      return;
    }

    runOperation(
      'loadWallet',
      () => NitroArk.loadWallet(ARK_DATA_PATH, getWalletConfig(mnemonic)),
      'management',
      () => {
        setResults((prev) => ({
          ...prev,
          management: 'Wallet loaded successfully!',
        }));
      }
    );
  };

  const handleCloseWallet = () => {
    runOperation('closeWallet', () => NitroArk.closeWallet(), 'management');
  };

  const handleRefreshServer = () => {
    runOperation(
      'refreshServer',
      () => NitroArk.refreshServer(),
      'management',
      () => {
        setResults((prev) => ({
          ...prev,
          management: 'Connection OK',
        }));
      }
    );
  };

  const handleIsWalletLoaded = () => {
    runOperation(
      'isWalletLoaded',
      () => NitroArk.isWalletLoaded(),
      'management'
    );
  };

  const handleMaintenance = () => {
    runOperation('maintenance', () => NitroArk.maintenance(), 'management');
  };

  const handleMaintenanceRefresh = () => {
    runOperation(
      'maintenanceRefresh',
      () => NitroArk.maintenanceRefresh(),
      'management'
    );
  };

  const handleSyncPendingBoards = () => {
    runOperation(
      'syncPendingBoards',
      () => NitroArk.syncPendingBoards(),
      'management'
    );
  };

  const handleMaintenanceWithOnchain = () => {
    runOperation(
      'maintenanceWithOnchain',
      () => NitroArk.maintenanceWithOnchain(),
      'management'
    );
  };

  const handleSync = () => {
    runOperation(
      'sync',
      async () => {
        const startTime = new Date().getTime();
        console.log('Starting sync at:', new Date(startTime).toISOString());
        await NitroArk.sync();

        const endTime = new Date().getTime();
        console.log('Finished sync at:', new Date(endTime).toISOString());

        const duration = (endTime - startTime) / 1000;
        console.log(`Sync took ${duration.toFixed(2)} seconds`);
      },
      'management'
    );
  };

  const handleOnchainSync = () => {
    runOperation(
      'onchainSync',
      async () => {
        const startTime = new Date().getTime();
        console.log(
          'Starting onchain sync at:',
          new Date(startTime).toISOString()
        );

        await NitroArk.onchainSync();

        const endTime = new Date().getTime();
        console.log(
          'Finished onchain sync at:',
          new Date(endTime).toISOString()
        );

        const duration = (endTime - startTime) / 1000;
        console.log(`Onchain sync took ${duration.toFixed(2)} seconds`);
      },
      'management'
    );
  };

  const handleSyncExits = () => {
    runOperation('syncExits', () => NitroArk.syncExits(), 'management');
  };

  const handleStartSyncPastRounds = () => {
    runOperation(
      'startSyncPastRounds',
      () => NitroArk.startSyncPastRounds(),
      'management'
    );
  };

  const handleSyncPendingRounds = () => {
    runOperation(
      'syncPendingRounds',
      () => NitroArk.syncPendingRounds(),
      'management'
    );
  };

  const handleGetArkInfo = () => {
    runOperation(
      'getArkInfo',
      () => NitroArk.getArkInfo(),
      'walletInfo',
      setArkInfo
    );
  };

  const handleGetOnchainBalance = () => {
    runOperation(
      'onchainBalance',
      () => NitroArk.onchainBalance(),
      'walletInfo',
      (balance) => {
        setOnchainBalance(balance);
        setResults((prev) => ({
          ...prev,
          walletInfo: `Onchain Balance: ${JSON.stringify(balance, null, 2)}`,
        }));
      }
    );
  };

  const handleGetOffchainBalance = () => {
    runOperation(
      'offchainBalance',
      () => NitroArk.offchainBalance(),
      'walletInfo',
      (balance) => {
        setOffchainBalance(balance);
        setResults((prev) => ({
          ...prev,
          walletInfo: `Offchain Balance: ${JSON.stringify(balance, null, 2)}`,
        }));
      }
    );
  };

  const handleDeriveStoreNextKeypair = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'deriveStoreNextKeypair',
      () => NitroArk.deriveStoreNextKeypair(),
      'walletInfo'
    );
  };

  const handlePeakKeyPair = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation('peakKeyPair', () => NitroArk.peakKeyPair(0), 'walletInfo');
  };

  const handleDeriveKeypairFromMnemonic = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'deriveKeypairFromMnemonic',
      () => NitroArk.deriveKeypairFromMnemonic(mnemonic, 'regtest', 0),
      'walletInfo'
    );
  };

  const handleNewAddress = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'newAddress',
      () => NitroArk.newAddress(),
      'walletInfo',
      (address) => {
        setResults((prev) => ({
          ...prev,
          walletInfo: JSON.stringify(address, null, 2),
        }));
      }
    );
  };

  const handlePeakAddress = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'peakAddress',
      () => NitroArk.peakAddress(0),
      'walletInfo',
      (address) => {
        setResults((prev) => ({
          ...prev,
          walletInfo: `Peak Address: ${JSON.stringify(address, null, 2)}`,
        }));
      }
    );
  };

  const handleGetOnchainAddress = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'onchainAddress',
      () => NitroArk.onchainAddress(),
      'walletInfo'
    );
  };

  const handleGetOnchainUtxos = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation('onchainUtxos', () => NitroArk.onchainUtxos(), 'walletInfo');
  };

  const handleGetVtxos = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation('vtxos', () => NitroArk.vtxos(), 'walletInfo');
  };

  const handleGetExpiringVtxos = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'getExpiringVtxos',
      () => NitroArk.getExpiringVtxos(5000),
      'walletInfo'
    );
  };

  const handleGetFirstExpiringVtxoBlockheight = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'getFirstExpiringVtxoBlockheight',
      () => NitroArk.getFirstExpiringVtxoBlockheight(),
      'walletInfo'
    );
  };

  const handleGetNextRequiredRefreshBlockheight = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'getNextRequiredRefreshBlockheight',
      () => NitroArk.getNextRequiredRefreshBlockheight(),
      'walletInfo'
    );
  };

  const handleGetMovements = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, walletInfo: 'Mnemonic required' }));
      return;
    }
    runOperation('movements', () => NitroArk.movements(), 'walletInfo');
  };

  const handleSendOnchain = () => {
    if (!onchainDestinationAddress || !onchainAmountSat) {
      setError((prev) => ({
        ...prev,
        onchain: 'Destination Address and Amount are required.',
      }));
      return;
    }
    const amountNum = parseInt(onchainAmountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError((prev) => ({ ...prev, onchain: 'Invalid amount specified.' }));
      return;
    }
    runOperation(
      'onchainSend',
      () => NitroArk.onchainSend(onchainDestinationAddress, amountNum),
      'onchain'
    );
  };

  const handleDrainOnchain = () => {
    if (!onchainDestinationAddress) {
      setError((prev) => ({
        ...prev,
        onchain: 'Destination Address is required.',
      }));
      return;
    }
    runOperation(
      'onchainDrain',
      () => NitroArk.onchainDrain(onchainDestinationAddress),
      'onchain'
    );
  };

  const handleSendManyOnchain = () => {
    if (!onchainDestinationAddress || !onchainAmountSat) {
      setError((prev) => ({
        ...prev,
        onchain:
          'At least one Destination Address and corresponding Amount are required.',
      }));
      return;
    }
    const amountNum = parseInt(onchainAmountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError((prev) => ({
        ...prev,
        onchain: 'Invalid amount specified for the first output.',
      }));
      return;
    }
    // Example: Using inputs for a single output in sendMany
    const outputs: BarkSendManyOutput[] = [
      { destination: onchainDestinationAddress, amountSat: amountNum },
      // Add more outputs here if needed, maybe from a more complex input UI
    ];
    runOperation(
      'onchainSendMany',
      () => NitroArk.onchainSendMany(outputs),
      'onchain'
    );
  };

  const handleBoardAmount = () => {
    if (!arkAmountSat) {
      setError((prev) => ({ ...prev, ark: 'Amount is required.' }));
      return;
    }
    const amountNum = parseInt(arkAmountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount specified.' }));
      return;
    }
    runOperation('boardAmount', () => NitroArk.boardAmount(amountNum), 'ark');
  };

  const handleBoardAll = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, ark: 'Mnemonic required' }));
      return;
    }
    runOperation('boardAll', () => NitroArk.boardAll(), 'ark');
  };

  const handleSendArkoorPayment = () => {
    if (!arkDestinationAddress || !arkAmountSat) {
      setError((prev) => ({
        ...prev,
        ark: 'Destination and Amount are required.',
      }));
      return;
    }
    const amountNum = parseInt(arkAmountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount specified.' }));
      return;
    }
    runOperation(
      'sendArkoorPayment',
      () => NitroArk.sendArkoorPayment(arkDestinationAddress, amountNum),
      'ark'
    );
  };

  const handleSendLightningPayment = () => {
    if (!arkDestinationAddress) {
      setError((prev) => ({
        ...prev,
        ark: 'Destination (invoice) is required.',
      }));
      return;
    }
    // Amount can be 0 to use invoice's amount
    const amountNum = parseInt(arkAmountSat, 10) || 0;
    if (isNaN(amountNum) || amountNum < 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount specified.' }));
      return;
    }
    runOperation(
      'payLightningInvoice',
      () => NitroArk.payLightningInvoice(arkDestinationAddress),
      'ark'
    );
  };

  const handlePayOffer = () => {
    if (!arkDestinationAddress) {
      setError((prev) => ({
        ...prev,
        ark: 'Destination (Bolt12 offer) is required.',
      }));
      return;
    }
    // Amount can be 0 to use offer's amount if specified
    const amountNum = parseInt(arkAmountSat, 10) || 0;
    if (isNaN(amountNum) || amountNum < 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount specified.' }));
      return;
    }
    runOperation(
      'payLightningOffer',
      () => NitroArk.payLightningOffer(arkDestinationAddress, amountNum),
      'ark'
    );
  };

  const handleSendLnaddr = () => {
    if (!arkDestinationAddress || !arkAmountSat) {
      setError((prev) => ({
        ...prev,
        ark: 'Destination (lnaddr) and Amount are required.',
      }));
      return;
    }
    const amountNum = parseInt(arkAmountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount specified.' }));
      return;
    }
    runOperation(
      'payLightningAddress',
      () =>
        NitroArk.payLightningAddress(
          arkDestinationAddress,
          amountNum,
          arkComment
        ),
      'ark'
    );
  };

  const handleSendRoundOnchainPayment = () => {
    if (!arkDestinationAddress || !arkAmountSat) {
      setError((prev) => ({
        ...prev,
        ark: 'Destination Address and Amount are required.',
      }));
      return;
    }
    const amountNum = parseInt(arkAmountSat, 10);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount specified.' }));
      return;
    }
    runOperation(
      'sendRoundOnchainPayment',
      () => NitroArk.sendRoundOnchainPayment(arkDestinationAddress, amountNum),
      'ark'
    );
  };

  const handleValidateArkoorAddress = () => {
    if (!arkoorAddressToValidate) {
      setError((prev) => ({
        ...prev,
        ark: 'Arkoor address to validate is required.',
      }));
      return;
    }
    runOperation(
      'validateArkoorAddress',
      () => NitroArk.validateArkoorAddress(arkoorAddressToValidate),
      'ark'
    );
  };

  const handleOffboardSpecific = () => {
    if (!vtxoIdsInput || !optionalAddress) {
      setError((prev) => ({
        ...prev,
        offboarding: 'VTXO IDs and Destination Address are required.',
      }));
      return;
    }
    const ids = vtxoIdsInput
      .split(',')
      .map((id) => id.trim())
      .filter((id) => id);
    if (ids.length === 0) {
      setError((prev) => ({
        ...prev,
        offboarding: 'At least one VTXO ID is required.',
      }));
      return;
    }
    runOperation(
      'offboardSpecific',
      () => NitroArk.offboardSpecific(ids, optionalAddress),
      'offboarding'
    );
  };

  const handleOffboardAll = () => {
    if (!optionalAddress) {
      setError((prev) => ({
        ...prev,
        offboarding: 'Destination Address is required.',
      }));
      return;
    }
    runOperation(
      'offboardAll',
      () => NitroArk.offboardAll(optionalAddress),
      'offboarding'
    );
  };

  const handleCreateInvoice = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, lightning: 'Mnemonic required' }));
      return;
    }
    const amount = parseInt(invoiceAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, lightning: 'Invalid amount specified.' }));
      return;
    }
    runOperation(
      'bolt11Invoice',
      () => NitroArk.bolt11Invoice(amount),
      'lightning',
      (invoice) => {
        setResults((prev) => ({
          ...prev,
          lightning: `Created Invoice: ${JSON.stringify(invoice)}`,
        }));
        setInvoiceToClaim(invoice);
      }
    );
  };

  const handleTryClaimLightningReceive = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, lightning: 'Mnemonic required' }));
      return;
    }
    if (!invoiceToClaim) {
      setError((prev) => ({
        ...prev,
        lightning: 'Invoice to claim is required.',
      }));
      return;
    }
    runOperation(
      'tryClaimLightningReceive',
      () => NitroArk.tryClaimLightningReceive(paymentHash, false),
      'lightning',
      () => {
        setResults((prev) => ({
          ...prev,
          lightning: 'Successfully claimed payment!',
        }));
      }
    );
  };

  const handleTryClaimAllLightningReceives = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, lightning: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'tryClaimAllLightningReceives',
      () => NitroArk.tryClaimAllLightningReceives(false),
      'lightning',
      () => {
        setResults((prev) => ({
          ...prev,
          lightning: 'Successfully claimed all open invoices!',
        }));
      }
    );
  };

  const handleLightningReceiveStatus = () => {
    if (!paymentHash) {
      setError((prev) => ({
        ...prev,
        lightning: 'Payment hash is required.',
      }));
      return;
    }
    runOperation(
      'lightningReceiveStatus',
      () => NitroArk.lightningReceiveStatus(paymentHash),
      'lightning'
    );
  };

  const handleSignMessage = () => {
    if (!messageToSign) {
      setError((prev) => ({
        ...prev,
        signing: 'Message is required to sign.',
      }));
      return;
    }
    runOperation(
      'signMessage',
      () => NitroArk.signMessage(messageToSign, 0),
      'signing',
      (sig) => {
        setSignature(sig);
        setResults((prev) => ({ ...prev, signing: `Signature: ${sig}` }));
      }
    );
  };

  const handleSignMesssageWithMnemonic = () => {
    if (!messageToSign || !mnemonic) {
      setError((prev) => ({
        ...prev,
        signing: 'Message and mnemonic are required to sign.',
      }));
      return;
    }
    runOperation(
      'signMesssageWithMnemonic',
      () =>
        NitroArk.signMesssageWithMnemonic(
          messageToSign,
          mnemonic,
          'regtest',
          0
        ),
      'signing',
      (sig) => {
        setSignature(sig);
        setResults((prev) => ({ ...prev, signing: `Signature: ${sig}` }));
      }
    );
  };

  const handleVerifyMessage = () => {
    if (!messageToSign || !signature || !publicKeyForVerification) {
      setError((prev) => ({
        ...prev,
        signing: 'Message, signature, and public key are required to verify.',
      }));
      return;
    }
    runOperation(
      'verifyMessage',
      () =>
        NitroArk.verifyMessage(
          messageToSign,
          signature,
          publicKeyForVerification
        ),
      'signing'
    );
  };

  // --- Render ---
  const canUseWallet = !!mnemonic;
  const walletOpsButtonDisabled = isLoading || !canUseWallet;

  const renderOperationButton = (title: string, onPress: () => void) => (
    <View style={styles.buttonWrapper}>
      <CustomButton
        title={title}
        onPress={onPress}
        disabled={walletOpsButtonDisabled}
      />
    </View>
  );

  const renderResult = (section: string) => {
    return (
      <>
        {results[section] && (
          <View style={styles.resultContainer}>
            <Text style={styles.resultHeader}>Result:</Text>
            <Text style={styles.resultText} selectable={true}>
              {results[section]}
            </Text>
          </View>
        )}
        {error[section] && (
          <View style={styles.errorContainer}>
            <Text style={styles.errorHeader}>Error:</Text>
            <Text style={styles.errorText} selectable={true}>
              {error[section]}
            </Text>
          </View>
        )}
      </>
    );
  };

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
        <View style={styles.operationSection}>
          <Text style={styles.sectionHeader}>Management</Text>
          {renderResult('management')}
          <View style={styles.buttonGrid}>
            <View style={styles.buttonWrapper}>
              <CustomButton
                title="Generate Mnemonic"
                onPress={handleCreateMnemonic}
                disabled={isLoading || !!mnemonic} // Disable if already generated
              />
            </View>
            <View style={styles.buttonWrapper}>
              <CustomButton
                title="Clear Mnemonic"
                onPress={handleClearMnemonic}
                disabled={isLoading || !mnemonic} // Disable if no mnemonic
                color="#ff6666" // Red color to indicate destructive action
              />
            </View>
            {renderOperationButton('Create Wallet', handleCreateWallet)}
            {renderOperationButton('Load Wallet', handleLoadWallet)}
            {renderOperationButton('Close Wallet', handleCloseWallet)}
            {renderOperationButton('Check Connection', handleRefreshServer)}
            <View style={styles.buttonWrapper}>
              <CustomButton
                title="Check Wallet Status"
                onPress={handleIsWalletLoaded}
                disabled={isLoading}
              />
            </View>
            {renderOperationButton('Maintenance', handleMaintenance)}
            {renderOperationButton(
              'Maintenance Refresh',
              handleMaintenanceRefresh
            )}
            {renderOperationButton(
              'Sync Pending Boards',
              handleSyncPendingBoards
            )}
            {renderOperationButton(
              'Maintenance With Onchain',
              handleMaintenanceWithOnchain
            )}
            {renderOperationButton('Sync', handleSync)}
            {renderOperationButton('Onchain Sync', handleOnchainSync)}
            {renderOperationButton('Sync Exits', handleSyncExits)}
            {renderOperationButton(
              'Start Sync Past Rounds',
              handleStartSyncPastRounds
            )}
            {renderOperationButton(
              'Sync Pending Rounds',
              handleSyncPendingRounds
            )}
          </View>
        </View>

        {/* --- Android Kotlin/JNI Demo --- */}
        {Platform.OS === 'android' && (
          <View style={styles.operationSection}>
            <Text style={styles.sectionHeader}>Android Kotlin/JNI Demo</Text>
            {renderResult('androidNative')}
            <View style={styles.buttonGrid}>
              {renderOperationButton(
                'JNI Load Wallet (Kotlin)',
                handleAndroidNativeLoadWallet
              )}
              {renderOperationButton(
                'JNI Is Wallet Loaded',
                handleAndroidNativeIsWalletLoaded
              )}
              {renderOperationButton(
                'JNI Close Wallet',
                handleAndroidNativeCloseWallet
              )}
              <View style={styles.buttonWrapper}>
                <CustomButton
                  title="JNI Maintenance (Kotlin)"
                  onPress={handleAndroidNativeMaintenance}
                />
              </View>
              {renderOperationButton(
                'JNI Maintenance Refresh',
                handleAndroidNativeMaintenanceRefresh
              )}
              {renderOperationButton(
                'JNI Try Claim Receive',
                handleAndroidNativeTryClaimLightningReceive
              )}
              {renderOperationButton(
                'JNI Offboard All',
                handleAndroidNativeOffboardAll
              )}
              {renderOperationButton(
                'JNI Peak Keypair',
                handleAndroidNativePeakKeyPair
              )}
              {renderOperationButton(
                'JNI Verify Message',
                handleAndroidNativeVerifyMessage
              )}
              {renderOperationButton(
                'JNI Bolt11 Invoice',
                handleAndroidNativeBolt11Invoice
              )}
              {renderOperationButton(
                'JNI Sign Message',
                handleAndroidNativeSignMessage
              )}
              {renderOperationButton('JNI Sync', handleAndroidNativeSync)}
            </View>
            <Text style={styles.statusText}>
              These buttons call a Kotlin module in the example app which
              forwards to the JNI shim inside the NitroArk library.
            </Text>
          </View>
        )}

        {/* --- Wallet Info --- */}
        <View style={styles.operationSection}>
          <Text style={styles.sectionHeader}>Wallet Info</Text>
          {renderResult('walletInfo')}
          <View style={styles.balanceContainer}>
            <Text style={styles.balanceHeader}>Wallet Balance</Text>
            <Text>
              Onchain (Confirmed): {formatSats(onchainBalance?.confirmed)}
            </Text>
            <Text>
              Onchain (Immature): {formatSats(onchainBalance?.immature)}
            </Text>
            <Text>
              Onchain (Pending):{' '}
              {formatSats(
                (onchainBalance?.trusted_pending ?? 0) +
                  (onchainBalance?.untrusted_pending ?? 0)
              )}
            </Text>
            <Text>
              Offchain (Spendable): {formatSats(offchainBalance?.spendable)}
            </Text>
            <Text>
              Offchain (Pending Send):{' '}
              {formatSats(offchainBalance?.pending_lightning_send)}
            </Text>
            <Text>
              Offchain (Pending Exit):{' '}
              {formatSats(offchainBalance?.pending_exit)}
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
          <View style={styles.buttonGrid}>
            {renderOperationButton('Get Ark Info', handleGetArkInfo)}
            {renderOperationButton(
              'Get Onchain Balance',
              handleGetOnchainBalance
            )}
            <View style={styles.buttonWrapper}>
              <CustomButton
                title="Get Offchain Balance"
                onPress={handleGetOffchainBalance}
                disabled={walletOpsButtonDisabled}
              />
            </View>
            {renderOperationButton(
              'Get Onchain Address',
              handleGetOnchainAddress
            )}
            {renderOperationButton(
              'Derive Store Next Keypair',
              handleDeriveStoreNextKeypair
            )}
            {renderOperationButton('Peak Key Pair', handlePeakKeyPair)}
            {renderOperationButton('Peak Address', handlePeakAddress)}
            {renderOperationButton(
              'Derive Keypair from Mnemonic',
              handleDeriveKeypairFromMnemonic
            )}
            {renderOperationButton(
              'Generate new Ark address',
              handleNewAddress
            )}
            {renderOperationButton('Get Onchain UTXOs', handleGetOnchainUtxos)}
            {renderOperationButton('VTXOs', handleGetVtxos)}
            {renderOperationButton(
              'Get Expiring VTXOs',
              handleGetExpiringVtxos
            )}
            {renderOperationButton('Get Movements', handleGetMovements)}
            {renderOperationButton(
              'Get First Expiring VTXO Blockheight',
              handleGetFirstExpiringVtxoBlockheight
            )}
            {renderOperationButton(
              'Get Next Required Refresh Blockheight',
              handleGetNextRequiredRefreshBlockheight
            )}
          </View>
        </View>

        {/* --- Onchain Operations --- */}
        <View style={styles.operationSection}>
          <Text style={styles.sectionHeader}>Onchain Operations</Text>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Destination Address:</Text>
            <TextInput
              style={styles.input}
              value={onchainDestinationAddress}
              onChangeText={setOnchainDestinationAddress}
              placeholder="Enter Bitcoin Address"
              autoCapitalize="none"
            />
          </View>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Amount (Satoshis):</Text>
            <TextInput
              style={styles.input}
              value={onchainAmountSat}
              onChangeText={setOnchainAmountSat}
              placeholder="e.g., 100000"
              keyboardType="numeric"
            />
          </View>
          <View style={styles.buttonGrid}>
            {renderOperationButton('Send Onchain', handleSendOnchain)}
            {renderOperationButton('Drain Onchain', handleDrainOnchain)}
            {renderOperationButton('Send Many Onchain', handleSendManyOnchain)}
          </View>
        </View>

        {/* --- Ark & Lightning Payments --- */}
        <View style={styles.operationSection}>
          <Text style={styles.sectionHeader}>Ark & Lightning Payments</Text>
          {renderResult('ark')}
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>
              Destination Address / Pubkey / Invoice:
            </Text>
            <TextInput
              style={styles.input}
              value={arkDestinationAddress}
              onChangeText={setArkDestinationAddress}
              placeholder="Enter destination"
              autoCapitalize="none"
            />
          </View>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Amount (Satoshis):</Text>
            <TextInput
              style={styles.input}
              value={arkAmountSat}
              onChangeText={setArkAmountSat}
              placeholder="e.g., 100000"
              keyboardType="numeric"
            />
          </View>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Comment (for Ark Send):</Text>
            <TextInput
              style={styles.input}
              value={arkComment}
              onChangeText={setArkComment}
              placeholder="Optional comment"
            />
          </View>
          <View style={styles.buttonGrid}>
            {renderOperationButton('Board Amount', handleBoardAmount)}
            {renderOperationButton('Board All', handleBoardAll)}
            {renderOperationButton(
              'Send Arkoor Payment',
              handleSendArkoorPayment
            )}
            {renderOperationButton(
              'Send Lightning Payment',
              handleSendLightningPayment
            )}
            {renderOperationButton('Pay Bolt12 Offer', handlePayOffer)}
            {renderOperationButton('Send to LN Address', handleSendLnaddr)}
            {renderOperationButton(
              'Send Round Onchain',
              handleSendRoundOnchainPayment
            )}
          </View>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Arkoor Address to Validate:</Text>
            <TextInput
              style={styles.input}
              value={arkoorAddressToValidate}
              onChangeText={setArkoorAddressToValidate}
              placeholder="Enter Arkoor address"
              autoCapitalize="none"
            />
          </View>
          <View style={styles.buttonGrid}>
            {renderOperationButton(
              'Validate Arkoor Address',
              handleValidateArkoorAddress
            )}
          </View>
        </View>

        {/* --- Lightning Operations --- */}
        <View style={styles.operationSection}>
          <Text style={styles.sectionHeader}>Lightning Operations</Text>
          {renderResult('lightning')}
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
          <View style={styles.buttonGrid}>
            {renderOperationButton('Create Invoice', handleCreateInvoice)}
          </View>

          <View style={styles.buttonGrid}>
            {renderOperationButton(
              'Try Claim Lightning Receive',
              handleTryClaimLightningReceive
            )}
          </View>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Payment Hash:</Text>
            <TextInput
              style={styles.input}
              value={paymentHash}
              onChangeText={setPaymentHash}
              placeholder="Enter payment hash"
              autoCapitalize="none"
            />
          </View>
          <View style={styles.buttonGrid}>
            {renderOperationButton(
              'Get Lightning Receive Status',
              handleLightningReceiveStatus
            )}
          </View>

          <View style={styles.buttonGrid}>
            {renderOperationButton(
              'Check and Claim All Open Ln Receives',
              handleTryClaimAllLightningReceives
            )}
          </View>
        </View>

        {/* --- Offboarding / Exiting --- */}
        <View style={styles.operationSection}>
          <Text style={styles.sectionHeader}>Offboarding / Exiting</Text>
          {renderResult('offboarding')}
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>VTXO IDs (Comma-separated):</Text>
            <TextInput
              style={styles.input}
              value={vtxoIdsInput}
              onChangeText={setVtxoIdsInput}
              placeholder="vtxo_id_1,vtxo_id_2,..."
              autoCapitalize="none"
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
            />
          </View>
          <View style={styles.buttonGrid}>
            {renderOperationButton('Offboard Specific', handleOffboardSpecific)}
            {renderOperationButton('Offboard All', handleOffboardAll)}
          </View>
        </View>

        {/* --- Signing/Verification --- */}
        <View style={styles.operationSection}>
          <Text style={styles.sectionHeader}>Signing & Verification</Text>
          {renderResult('signing')}
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Message to Sign/Verify:</Text>
            <TextInput
              style={styles.input}
              value={messageToSign}
              onChangeText={setMessageToSign}
              placeholder="Enter message"
            />
          </View>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Public Key for Verification:</Text>
            <TextInput
              style={styles.input}
              value={publicKeyForVerification}
              onChangeText={setPublicKeyForVerification}
              placeholder="Enter public key"
              autoCapitalize="none"
            />
          </View>
          <View style={styles.buttonGrid}>
            {renderOperationButton('Sign Message (key 0)', handleSignMessage)}
            {renderOperationButton('Verify Message', handleVerifyMessage)}
          </View>
          <View style={styles.buttonGrid}>
            {renderOperationButton(
              'Sign Message (with mnemonic)',
              handleSignMesssageWithMnemonic
            )}
          </View>
          <View style={styles.inputContainer}>
            <Text style={styles.inputLabel}>Signature:</Text>
            <TextInput
              style={styles.input}
              value={signature}
              onChangeText={setSignature}
              placeholder="Signature will appear here"
            />
          </View>
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
    marginTop: 15,
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
  inputContainer: {
    marginVertical: 8,
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
  buttonGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
  },
  buttonWrapper: {
    width: '48%', // Two columns with a small gap
    marginVertical: 5,
  },
  operationSection: {
    marginVertical: 10,
    padding: 10,
    backgroundColor: '#ffffff',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#ddd',
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
  customButton: {
    paddingVertical: 12,
    paddingHorizontal: 10,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 44,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  customButtonDisabled: {
    opacity: 0.5,
  },
  customButtonText: {
    color: 'white',
    fontSize: 14,
    fontWeight: '600',
    textAlign: 'center',
  },
});
