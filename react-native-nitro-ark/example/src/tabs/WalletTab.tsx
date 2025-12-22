import { useState } from 'react';
import { View, Text, ScrollView, StyleSheet } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import RNFSTurbo from 'react-native-fs-turbo';
import * as NitroArk from 'react-native-nitro-ark';

import {
  CustomButton,
  InputField,
  ResultBox,
  Section,
  ButtonGrid,
  BalanceCard,
  InfoRow,
} from '../components';
import {
  COLORS,
  ARK_DATA_PATH,
  MNEMONIC_STORAGE_KEY,
  getWalletConfig,
  formatSats,
} from '../constants';
import type { TabProps } from '../types';

export const WalletTab = ({
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
  isWalletLoaded,
  setIsWalletLoaded,
  runOperation,
}: TabProps) => {
  const [messageToSign, setMessageToSign] = useState('hello world');
  const [signature, setSignature] = useState('');
  const [publicKeyForVerification, setPublicKeyForVerification] = useState('');

  const canUseWallet = !!mnemonic;
  const walletOpsDisabled = isLoading || !canUseWallet;

  // --- Wallet Management ---
  const handleCreateMnemonic = () => {
    runOperation(
      'createMnemonic',
      () => NitroArk.createMnemonic(),
      'wallet',
      async (newMnemonic) => {
        setMnemonic(newMnemonic);
        try {
          await AsyncStorage.setItem(MNEMONIC_STORAGE_KEY, newMnemonic);
          setResults((prev) => ({
            ...prev,
            wallet: `Mnemonic created and saved!\n\n${newMnemonic}`,
          }));
        } catch (err: any) {
          setError((prev) => ({
            ...prev,
            wallet: 'Failed to save mnemonic: ' + err.message,
          }));
        }
      }
    );
  };

  const handleClearMnemonic = async () => {
    try {
      await AsyncStorage.removeItem(MNEMONIC_STORAGE_KEY);
      RNFSTurbo.unlink(ARK_DATA_PATH);
      setMnemonic(undefined);
      setArkInfo(undefined);
      setOnchainBalance(undefined);
      setOffchainBalance(undefined);
      setIsWalletLoaded(false);
      setResults((prev) => ({ ...prev, wallet: 'Wallet data cleared!' }));
    } catch (err: any) {
      setError((prev) => ({
        ...prev,
        wallet: 'Failed to clear: ' + err.message,
      }));
    }
  };

  const handleCreateWallet = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, wallet: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'createWallet',
      () => NitroArk.createWallet(ARK_DATA_PATH, getWalletConfig(mnemonic)),
      'wallet',
      () => setResults((prev) => ({ ...prev, wallet: 'Wallet created!' }))
    );
  };

  const handleLoadWallet = () => {
    if (!mnemonic) {
      setError((prev) => ({ ...prev, wallet: 'Mnemonic required' }));
      return;
    }
    runOperation(
      'loadWallet',
      () => NitroArk.loadWallet(ARK_DATA_PATH, getWalletConfig(mnemonic)),
      'wallet',
      () => {
        setIsWalletLoaded(true);
        setResults((prev) => ({ ...prev, wallet: 'Wallet loaded!' }));
      }
    );
  };

  const handleCloseWallet = () => {
    runOperation(
      'closeWallet',
      () => NitroArk.closeWallet(),
      'wallet',
      () => {
        setIsWalletLoaded(false);
        setResults((prev) => ({ ...prev, wallet: 'Wallet closed!' }));
      }
    );
  };

  const handleIsWalletLoaded = () => {
    runOperation(
      'isWalletLoaded',
      () => NitroArk.isWalletLoaded(),
      'wallet',
      (loaded: boolean) => {
        setIsWalletLoaded(loaded);
        setResults((prev) => ({
          ...prev,
          wallet: loaded ? 'Wallet is loaded' : 'Wallet is not loaded',
        }));
      }
    );
  };

  // --- Sync Operations ---
  const handleRefreshServer = () => {
    runOperation(
      'refreshServer',
      () => NitroArk.refreshServer(),
      'sync',
      () => setResults((prev) => ({ ...prev, sync: 'Server connection OK!' }))
    );
  };

  const handleSync = () => {
    runOperation(
      'sync',
      async () => {
        const start = Date.now();
        await NitroArk.sync();
        return `Sync completed in ${((Date.now() - start) / 1000).toFixed(2)}s`;
      },
      'sync'
    );
  };

  const handleOnchainSync = () => {
    runOperation(
      'onchainSync',
      async () => {
        const start = Date.now();
        await NitroArk.onchainSync();
        return `Onchain sync completed in ${((Date.now() - start) / 1000).toFixed(2)}s`;
      },
      'sync'
    );
  };

  const handleMaintenance = () => {
    runOperation(
      'maintenance',
      () => NitroArk.maintenance(),
      'sync',
      () => setResults((prev) => ({ ...prev, sync: 'Maintenance completed!' }))
    );
  };

  const handleMaintenanceRefresh = () => {
    runOperation(
      'maintenanceRefresh',
      () => NitroArk.maintenanceRefresh(),
      'sync',
      () =>
        setResults((prev) => ({ ...prev, sync: 'Maintenance refresh done!' }))
    );
  };

  const handleMaintenanceWithOnchain = () => {
    runOperation(
      'maintenanceWithOnchain',
      () => NitroArk.maintenanceWithOnchain(),
      'sync',
      () =>
        setResults((prev) => ({
          ...prev,
          sync: 'Maintenance with onchain done!',
        }))
    );
  };

  const handleSyncPendingBoards = () => {
    runOperation(
      'syncPendingBoards',
      () => NitroArk.syncPendingBoards(),
      'sync',
      () => setResults((prev) => ({ ...prev, sync: 'Pending boards synced!' }))
    );
  };

  const handleSyncExits = () => {
    runOperation(
      'syncExits',
      () => NitroArk.syncExits(),
      'sync',
      () => setResults((prev) => ({ ...prev, sync: 'Exits synced!' }))
    );
  };

  const handleSyncPendingRounds = () => {
    runOperation(
      'syncPendingRounds',
      () => NitroArk.syncPendingRounds(),
      'sync',
      () => setResults((prev) => ({ ...prev, sync: 'Pending rounds synced!' }))
    );
  };

  // --- Wallet Info ---
  const handleGetArkInfo = () => {
    runOperation('getArkInfo', () => NitroArk.getArkInfo(), 'info', setArkInfo);
  };

  const handleGetOnchainBalance = () => {
    runOperation(
      'onchainBalance',
      () => NitroArk.onchainBalance(),
      'info',
      setOnchainBalance
    );
  };

  const handleGetOffchainBalance = () => {
    runOperation(
      'offchainBalance',
      () => NitroArk.offchainBalance(),
      'info',
      setOffchainBalance
    );
  };

  const handleDeriveStoreNextKeypair = () => {
    runOperation(
      'deriveStoreNextKeypair',
      () => NitroArk.deriveStoreNextKeypair(),
      'info'
    );
  };

  const handlePeakKeyPair = () => {
    runOperation('peakKeyPair', () => NitroArk.peakKeyPair(0), 'info');
  };

  const handleDeriveKeypairFromMnemonic = () => {
    if (!mnemonic) return;
    runOperation(
      'deriveKeypairFromMnemonic',
      () => NitroArk.deriveKeypairFromMnemonic(mnemonic, 'regtest', 0),
      'info'
    );
  };

  const handleGetVtxos = () => {
    runOperation('vtxos', () => NitroArk.vtxos(), 'info');
  };

  const handleGetOnchainUtxos = () => {
    runOperation('onchainUtxos', () => NitroArk.onchainUtxos(), 'info');
  };

  const handleGetExpiringVtxos = () => {
    runOperation(
      'getExpiringVtxos',
      () => NitroArk.getExpiringVtxos(5000),
      'info'
    );
  };

  const handleGetFirstExpiringVtxoBlockheight = () => {
    runOperation(
      'getFirstExpiringVtxoBlockheight',
      () => NitroArk.getFirstExpiringVtxoBlockheight(),
      'info'
    );
  };

  const handleGetNextRequiredRefreshBlockheight = () => {
    runOperation(
      'getNextRequiredRefreshBlockheight',
      () => NitroArk.getNextRequiredRefreshBlockheight(),
      'info'
    );
  };

  const handleGetHistory = () => {
    runOperation('history', () => NitroArk.history(), 'info');
  };

  // --- Signing ---
  const handleSignMessage = () => {
    if (!messageToSign) {
      setError((prev) => ({ ...prev, signing: 'Message required' }));
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
        signing: 'Message and mnemonic required',
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
        signing: 'Message, signature, and public key required',
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

  return (
    <ScrollView style={styles.container} showsVerticalScrollIndicator={false}>
      {/* Status Bar */}
      <View style={styles.statusBar}>
        <View style={styles.statusRow}>
          <View
            style={[
              styles.statusDot,
              mnemonic ? styles.statusActive : styles.statusInactive,
            ]}
          />
          <Text style={styles.statusText}>
            {mnemonic ? 'Mnemonic loaded' : 'No mnemonic'}
          </Text>
        </View>
        <View style={styles.statusRow}>
          <View
            style={[
              styles.statusDot,
              isWalletLoaded ? styles.statusActive : styles.statusInactive,
            ]}
          />
          <Text style={styles.statusText}>
            {isWalletLoaded ? 'Wallet loaded' : 'Wallet not loaded'}
          </Text>
        </View>
      </View>

      {/* Wallet Management */}
      <Section title="Wallet Management">
        <ButtonGrid>
          <CustomButton
            title="Create Mnemonic"
            onPress={handleCreateMnemonic}
            disabled={isLoading}
          />
          <CustomButton
            title="Clear Data"
            onPress={handleClearMnemonic}
            disabled={isLoading}
            color={COLORS.danger}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Create Wallet"
            onPress={handleCreateWallet}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
          <CustomButton
            title="Load Wallet"
            onPress={handleLoadWallet}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Close Wallet"
            onPress={handleCloseWallet}
            disabled={isLoading}
          />
          <CustomButton
            title="Is Loaded?"
            onPress={handleIsWalletLoaded}
            disabled={isLoading}
          />
        </ButtonGrid>
        <ResultBox result={results.wallet} error={error.wallet} />
      </Section>

      {/* Sync Operations */}
      <Section title="Sync Operations">
        <ButtonGrid>
          <CustomButton
            title="Refresh Server"
            onPress={handleRefreshServer}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Sync"
            onPress={handleSync}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Onchain Sync"
            onPress={handleOnchainSync}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Maintenance"
            onPress={handleMaintenance}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Maintenance Refresh"
            onPress={handleMaintenanceRefresh}
            disabled={walletOpsDisabled}
            small
          />
          <CustomButton
            title="Maint. + Onchain"
            onPress={handleMaintenanceWithOnchain}
            disabled={walletOpsDisabled}
            small
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Sync Pending Boards"
            onPress={handleSyncPendingBoards}
            disabled={walletOpsDisabled}
            small
          />
          <CustomButton
            title="Sync Exits"
            onPress={handleSyncExits}
            disabled={walletOpsDisabled}
            small
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Sync Pending Rounds"
            onPress={handleSyncPendingRounds}
            disabled={walletOpsDisabled}
            small
          />
        </ButtonGrid>
        <ResultBox result={results.sync} error={error.sync} />
      </Section>

      {/* Balances */}
      <Section title="Balances">
        <ButtonGrid>
          <CustomButton
            title="Get Ark Info"
            onPress={handleGetArkInfo}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Offchain Balance"
            onPress={handleGetOffchainBalance}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Onchain Balance"
            onPress={handleGetOnchainBalance}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>

        {offchainBalance && (
          <BalanceCard
            title="Offchain (Ark)"
            balances={[
              {
                label: 'Spendable',
                value: formatSats(offchainBalance.spendable),
              },
              {
                label: 'Pending LN Send',
                value: formatSats(offchainBalance.pending_lightning_send),
              },
              {
                label: 'Pending Round',
                value: formatSats(offchainBalance.pending_in_round),
              },
              {
                label: 'Pending Exit',
                value: formatSats(offchainBalance.pending_exit),
              },
              {
                label: 'Pending Board',
                value: formatSats(offchainBalance.pending_board),
              },
            ]}
          />
        )}

        {onchainBalance && (
          <BalanceCard
            title="Onchain"
            balances={[
              {
                label: 'Confirmed',
                value: formatSats(onchainBalance.confirmed),
              },
              {
                label: 'Trusted Pending',
                value: formatSats(onchainBalance.trusted_pending),
              },
              {
                label: 'Untrusted Pending',
                value: formatSats(onchainBalance.untrusted_pending),
              },
              { label: 'Immature', value: formatSats(onchainBalance.immature) },
            ]}
          />
        )}

        {arkInfo && (
          <View style={styles.arkInfoContainer}>
            <Text style={styles.arkInfoTitle}>Ark Info</Text>
            <InfoRow label="Network" value={arkInfo.network} />
            <InfoRow label="Server Pubkey" value={arkInfo.server_pubkey} />
            <InfoRow
              label="Round Interval"
              value={`${arkInfo.round_interval}s`}
            />
            <InfoRow
              label="Max VTXO Amount"
              value={formatSats(arkInfo.max_vtxo_amount)}
            />
          </View>
        )}
      </Section>

      {/* Keys */}
      <Section title="Keys">
        <ButtonGrid>
          <CustomButton
            title="Derive Next Keypair"
            onPress={handleDeriveStoreNextKeypair}
            disabled={walletOpsDisabled}
            small
          />
          <CustomButton
            title="Peak Keypair"
            onPress={handlePeakKeyPair}
            disabled={walletOpsDisabled}
            small
          />
          <CustomButton
            title="Derive from Mnemonic"
            onPress={handleDeriveKeypairFromMnemonic}
            disabled={walletOpsDisabled}
            small
          />
        </ButtonGrid>
        <ResultBox result={results.info} error={error.info} />
      </Section>

      {/* UTXOs & VTXOs */}
      <Section title="UTXOs & VTXOs">
        <ButtonGrid>
          <CustomButton
            title="Get VTXOs"
            onPress={handleGetVtxos}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Get Onchain UTXOs"
            onPress={handleGetOnchainUtxos}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Expiring VTXOs"
            onPress={handleGetExpiringVtxos}
            disabled={walletOpsDisabled}
            small
          />
          <CustomButton
            title="First Expiry Height"
            onPress={handleGetFirstExpiringVtxoBlockheight}
            disabled={walletOpsDisabled}
            small
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Next Refresh Height"
            onPress={handleGetNextRequiredRefreshBlockheight}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="History"
            onPress={handleGetHistory}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
      </Section>

      {/* Message Signing */}
      <Section title="Message Signing">
        <InputField
          label="Message to Sign"
          value={messageToSign}
          onChangeText={setMessageToSign}
          placeholder="Enter message"
        />
        <InputField
          label="Signature (for verification)"
          value={signature}
          onChangeText={setSignature}
          placeholder="Signature will appear here"
        />
        <InputField
          label="Public Key (for verification)"
          value={publicKeyForVerification}
          onChangeText={setPublicKeyForVerification}
          placeholder="Enter public key"
        />
        <ButtonGrid>
          <CustomButton
            title="Sign"
            onPress={handleSignMessage}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Sign with Mnemonic"
            onPress={handleSignMesssageWithMnemonic}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Verify"
            onPress={handleVerifyMessage}
            disabled={isLoading}
          />
        </ButtonGrid>
        <ResultBox result={results.signing} error={error.signing} />
      </Section>

      <View style={styles.bottomPadding} />
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: COLORS.background,
    padding: 16,
  },
  statusBar: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    padding: 12,
    backgroundColor: COLORS.surface,
    borderRadius: 8,
    marginBottom: 16,
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusDot: {
    width: 10,
    height: 10,
    borderRadius: 5,
    marginRight: 8,
  },
  statusActive: {
    backgroundColor: COLORS.success,
  },
  statusInactive: {
    backgroundColor: COLORS.danger,
  },
  statusText: {
    color: COLORS.text,
    fontSize: 14,
  },
  arkInfoContainer: {
    backgroundColor: COLORS.surfaceLight,
    borderRadius: 8,
    padding: 12,
  },
  arkInfoTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: COLORS.text,
    marginBottom: 8,
  },
  bottomPadding: {
    height: 40,
  },
});
