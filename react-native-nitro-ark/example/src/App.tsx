import { useEffect, useState } from 'react';
import {
  Text,
  View,
  StyleSheet,
  Button,
  ScrollView,
  Platform,
  SafeAreaView,
} from 'react-native';
import {
  DocumentDirectoryPath,
  exists,
  mkdir,
} from '@dr.pogodin/react-native-fs';
import { createWallet, getBalance } from 'react-native-nitro-ark';

// Constants
const ARK_DATA_PATH = `${DocumentDirectoryPath}/bark_data`;

interface WalletState {
  isInitialized: boolean;
  errorMessage: string | undefined;
}

interface BalanceState {
  onchain: number;
  offchain: number;
  pendingExit: number;
  error: string | undefined;
}

export default function ArkApp() {
  const [walletState, setWalletState] = useState<WalletState>({
    isInitialized: false,
    errorMessage: undefined,
  });
  
  const [balanceState, setBalanceState] = useState<BalanceState>({
    onchain: 0,
    offchain: 0,
    pendingExit: 0,
    error: undefined,
  });

  useEffect(() => {
    const initWallet = async () => {
      try {
        // Ensure directory exists
        await ensureDataDirectory();

        // Create wallet if it doesn't exist
        const result = await createWallet(ARK_DATA_PATH, {
          force: true,
          regtest: false,
          signet: true,
          bitcoin: false,
          config: {
            esplora: "esplora.signet.2nd.dev",
            asp: "ark.signet.2nd.dev"
          },
        });

        if (result) {
          console.log('Wallet created successfully');
          setWalletState({
            isInitialized: true,
            errorMessage: undefined,
          });
          
          // Fetch initial balance
          await fetchBalance();
        } else {
          throw new Error('Failed to create wallet');
        }
      } catch (error: any) {
        console.error('Error initializing wallet:', error);
        setWalletState({
          isInitialized: false,
          errorMessage: error.message,
        });
      }
    };

    initWallet();
  }, []);

  const ensureDataDirectory = async () => {
    try {
      const dirExists = await exists(ARK_DATA_PATH);
      if (!dirExists) {
        await mkdir(ARK_DATA_PATH, {
          NSURLIsExcludedFromBackupKey: true, // iOS specific
        });
      }
    } catch (error: any) {
      console.error('Error with directory setup:', error);
      throw new Error(`Failed to setup data directory: ${error.message}`);
    }
  };

  const fetchBalance = async (skipSync: boolean = false) => {
    try {
      const balance = await getBalance(ARK_DATA_PATH, skipSync);
      console.log('Balance result:', balance);
      
      setBalanceState({
        onchain: balance.onchain,
        offchain: balance.offchain,
        pendingExit: balance.pending_exit,
        error: undefined,
      });
    } catch (err: any) {
      console.error('Error fetching balance:', err);
      setBalanceState(prev => ({
        ...prev,
        error: err.message,
      }));
    }
  };

  const formatSats = (sats: number): string => {
    return `${sats.toLocaleString()} sats (${(sats / 100000000).toFixed(8)} BTC)`;
  };

  return (
    <SafeAreaView style={styles.scrollContainer}>
      <ScrollView>
        <View style={styles.container}>
          <Text style={styles.headerText}>Bark Wallet Status</Text>

          <Text style={styles.statusText}>
            Wallet Initialized: {String(walletState.isInitialized)}
          </Text>
          
          {!walletState.isInitialized && walletState.errorMessage && (
            <Text style={styles.errorText}>Error: {walletState.errorMessage}</Text>
          )}

          <View style={styles.balanceContainer}>
            <Text style={styles.balanceHeader}>Wallet Balance</Text>
            <Text style={styles.balanceText}>Onchain: {formatSats(balanceState.onchain)}</Text>
            <Text style={styles.balanceText}>Offchain: {formatSats(balanceState.offchain)}</Text>
            <Text style={styles.balanceText}>Pending Exit: {formatSats(balanceState.pendingExit)}</Text>
            
            {balanceState.error && (
              <Text style={styles.errorText}>Error: {balanceState.error}</Text>
            )}
          </View>

          <View style={styles.buttonContainer}>
            <Button 
              title="Refresh Balance (with sync)" 
              onPress={() => fetchBalance(false)} 
            />
            <View style={styles.buttonSpacer} />
            <Button 
              title="Refresh Balance (skip sync)" 
              onPress={() => fetchBalance(true)} 
            />
          </View>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  scrollContainer: {
    flex: 1,
  },
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
    paddingTop: Platform.OS === 'ios' ? 50 : 20,
  },
  headerText: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 20,
  },
  statusText: {
    fontSize: 16,
    marginVertical: 8,
    textAlign: 'center',
  },
  balanceContainer: {
    width: '100%',
    marginVertical: 20,
    padding: 15,
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 8,
    backgroundColor: '#f9f9f9',
  },
  balanceHeader: {
    fontSize: 18,
    fontWeight: 'bold',
    marginBottom: 10,
    textAlign: 'center',
  },
  balanceText: {
    fontSize: 16,
    marginVertical: 5,
  },
  buttonContainer: {
    width: '100%',
    marginVertical: 16,
  },
  buttonSpacer: {
    height: 10,
  },
  errorText: {
    fontSize: 14,
    color: 'red',
    marginVertical: 4,
  },
});