import { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  SafeAreaView,
  TouchableOpacity,
  Platform,
} from 'react-native';
import RNFSTurbo from 'react-native-fs-turbo';
import AsyncStorage from '@react-native-async-storage/async-storage';

import type {
  BarkArkInfo,
  OnchainBalanceResult,
  OffchainBalanceResult,
} from 'react-native-nitro-ark';

import { WalletTab } from './tabs/WalletTab';
import { SendTab } from './tabs/SendTab';
import { ReceiveTab } from './tabs/ReceiveTab';
import { LoadingOverlay } from './components';
import { COLORS, ARK_DATA_PATH, MNEMONIC_STORAGE_KEY } from './constants';
import type { TabName } from './types';

export default function App() {
  // Tab state
  const [activeTab, setActiveTab] = useState<TabName>('wallet');

  // Wallet state
  const [mnemonic, setMnemonic] = useState<string | undefined>(undefined);
  const [arkInfo, setArkInfo] = useState<BarkArkInfo | undefined>();
  const [onchainBalance, setOnchainBalance] = useState<
    OnchainBalanceResult | undefined
  >();
  const [offchainBalance, setOffchainBalance] = useState<
    OffchainBalanceResult | undefined
  >();

  // Wallet loaded state
  const [isWalletLoaded, setIsWalletLoaded] = useState(false);

  // UI state
  const [results, setResults] = useState<{ [key: string]: string }>({});
  const [error, setError] = useState<{ [key: string]: string }>({});
  const [isLoading, setIsLoading] = useState(false);

  // Setup data directory on mount
  useEffect(() => {
    const setupDirectory = async () => {
      try {
        const dirExists = RNFSTurbo.exists(ARK_DATA_PATH);
        if (!dirExists) {
          RNFSTurbo.mkdir(ARK_DATA_PATH, {
            NSURLIsExcludedFromBackupKey: true,
          });
          console.log('Data directory created:', ARK_DATA_PATH);
        }
      } catch (err) {
        console.error('Error setting up data directory:', err);
      }
    };
    setupDirectory();
  }, []);

  // Load saved mnemonic on mount
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

  // Generic operation runner
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
      console.log(`Running: ${operationName}...`);

      try {
        const result = await operationFn();
        console.log(`${operationName} success:`, result);

        if (updateStateFn) {
          updateStateFn(result);
        } else {
          setResults((prev) => ({
            ...prev,
            [section]:
              typeof result === 'object' || typeof result === 'undefined'
                ? (JSON.stringify(result, null, 2) ??
                  'Success (no return value)')
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
    []
  );

  // Tab props shared across all tabs
  const tabProps = {
    mnemonic,
    setMnemonic,
    arkInfo,
    setArkInfo,
    onchainBalance,
    setOnchainBalance,
    offchainBalance,
    setOffchainBalance,
    results,
    setResults,
    error,
    setError,
    isLoading,
    setIsLoading,
    isWalletLoaded,
    setIsWalletLoaded,
    runOperation,
  };

  const renderTab = () => {
    switch (activeTab) {
      case 'wallet':
        return <WalletTab {...tabProps} />;
      case 'send':
        return <SendTab {...tabProps} />;
      case 'receive':
        return <ReceiveTab {...tabProps} />;
      default:
        return <WalletTab {...tabProps} />;
    }
  };

  return (
    <SafeAreaView style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <Text style={styles.headerTitle}>Nitro Ark</Text>
        <View style={styles.headerStatusContainer}>
          <View style={styles.headerStatusRow}>
            <View
              style={[
                styles.statusDot,
                mnemonic ? styles.statusActive : styles.statusInactive,
              ]}
            />
            <Text style={styles.statusLabel}>
              {mnemonic ? 'Mnemonic' : 'No Mnemonic'}
            </Text>
          </View>
          <View style={styles.headerStatusRow}>
            <View
              style={[
                styles.statusDot,
                isWalletLoaded ? styles.statusActive : styles.statusInactive,
              ]}
            />
            <Text style={styles.statusLabel}>
              {isWalletLoaded ? 'Wallet Loaded' : 'Not Loaded'}
            </Text>
          </View>
        </View>
      </View>

      {/* Tab Content */}
      <View style={styles.content}>{renderTab()}</View>

      {/* Bottom Tabs */}
      <View style={styles.tabBar}>
        <TabButton
          title="Wallet"
          icon="💼"
          isActive={activeTab === 'wallet'}
          onPress={() => setActiveTab('wallet')}
        />
        <TabButton
          title="Send"
          icon="📤"
          isActive={activeTab === 'send'}
          onPress={() => setActiveTab('send')}
        />
        <TabButton
          title="Receive"
          icon="📥"
          isActive={activeTab === 'receive'}
          onPress={() => setActiveTab('receive')}
        />
      </View>

      {/* Loading Overlay */}
      <LoadingOverlay visible={isLoading} />
    </SafeAreaView>
  );
}

interface TabButtonProps {
  title: string;
  icon: string;
  isActive: boolean;
  onPress: () => void;
}

const TabButton = ({ title, icon, isActive, onPress }: TabButtonProps) => (
  <TouchableOpacity
    style={[styles.tabButton, isActive && styles.tabButtonActive]}
    onPress={onPress}
  >
    <Text style={styles.tabIcon}>{icon}</Text>
    <Text style={[styles.tabLabel, isActive && styles.tabLabelActive]}>
      {title}
    </Text>
  </TouchableOpacity>
);

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: COLORS.background,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingVertical: 16,
    backgroundColor: COLORS.surface,
    borderBottomWidth: 1,
    borderBottomColor: COLORS.border,
  },
  headerTitle: {
    fontSize: 24,
    fontWeight: '700',
    color: COLORS.text,
  },
  headerStatusContainer: {
    alignItems: 'flex-end',
  },
  headerStatusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 2,
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 6,
  },
  statusActive: {
    backgroundColor: COLORS.success,
  },
  statusInactive: {
    backgroundColor: COLORS.textMuted,
  },
  statusLabel: {
    fontSize: 13,
    color: COLORS.textSecondary,
  },
  content: {
    flex: 1,
  },
  tabBar: {
    flexDirection: 'row',
    backgroundColor: COLORS.surface,
    borderTopWidth: 1,
    borderTopColor: COLORS.border,
    paddingBottom: Platform.OS === 'ios' ? 20 : 8,
    paddingTop: 8,
  },
  tabButton: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 8,
  },
  tabButtonActive: {
    backgroundColor: 'rgba(99, 102, 241, 0.1)',
    borderRadius: 8,
    marginHorizontal: 4,
  },
  tabIcon: {
    fontSize: 20,
    marginBottom: 4,
  },
  tabLabel: {
    fontSize: 12,
    color: COLORS.textMuted,
    fontWeight: '500',
  },
  tabLabelActive: {
    color: COLORS.primary,
    fontWeight: '600',
  },
});
