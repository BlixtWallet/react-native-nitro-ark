import { useState } from 'react';
import { View, ScrollView, StyleSheet, Text } from 'react-native';
import * as NitroArk from 'react-native-nitro-ark';

import {
  CustomButton,
  InputField,
  ResultBox,
  Section,
  ButtonGrid,
} from '../components';
import { COLORS } from '../constants';
import type { TabProps } from '../types';

export const ReceiveTab = ({
  mnemonic,
  results,
  setResults,
  error,
  setError,
  isLoading,
  runOperation,
}: TabProps) => {
  const [invoiceAmount, setInvoiceAmount] = useState('1000');
  const [paymentHash, setPaymentHash] = useState('');
  const [claimToken, setClaimToken] = useState('');
  const [lastInvoice, setLastInvoice] = useState<{
    invoice: string;
    paymentHash: string;
  } | null>(null);

  const canUseWallet = !!mnemonic;
  const walletOpsDisabled = isLoading || !canUseWallet;

  // --- Create Invoice ---
  const handleCreateInvoice = () => {
    const amount = parseInt(invoiceAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, invoice: 'Valid amount required' }));
      return;
    }
    runOperation(
      'bolt11Invoice',
      () => NitroArk.bolt11Invoice(amount),
      'invoice',
      (invoice) => {
        setLastInvoice({
          invoice: invoice.payment_request,
          paymentHash: invoice.payment_hash,
        });
        setPaymentHash(invoice.payment_hash);
        setResults((prev) => ({
          ...prev,
          invoice: `Invoice created!\n\nPayment Request:\n${invoice.payment_request}\n\nPayment Hash:\n${invoice.payment_hash}`,
        }));
      }
    );
  };

  // --- Lightning Receive Status ---
  const handleLightningReceiveStatus = () => {
    if (!paymentHash) {
      setError((prev) => ({ ...prev, status: 'Payment hash required' }));
      return;
    }
    runOperation(
      'lightningReceiveStatus',
      () => NitroArk.lightningReceiveStatus(paymentHash),
      'status'
    );
  };

  // --- Claim Lightning Receive ---
  const handleTryClaimLightningReceive = () => {
    if (!paymentHash) {
      setError((prev) => ({ ...prev, claim: 'Payment hash required' }));
      return;
    }
    const token = claimToken.trim() || undefined;
    runOperation(
      'tryClaimLightningReceive',
      () => NitroArk.tryClaimLightningReceive(paymentHash, false, token),
      'claim',
      () =>
        setResults((prev) => ({
          ...prev,
          claim: 'Successfully claimed payment!',
        }))
    );
  };

  const handleTryClaimLightningReceiveWait = () => {
    if (!paymentHash) {
      setError((prev) => ({ ...prev, claim: 'Payment hash required' }));
      return;
    }
    const token = claimToken.trim() || undefined;
    runOperation(
      'tryClaimLightningReceive (wait)',
      () => NitroArk.tryClaimLightningReceive(paymentHash, true, token),
      'claim',
      () =>
        setResults((prev) => ({
          ...prev,
          claim: 'Successfully claimed payment!',
        }))
    );
  };

  const handleTryClaimAllLightningReceives = () => {
    runOperation(
      'tryClaimAllLightningReceives',
      () => NitroArk.tryClaimAllLightningReceives(false),
      'claim',
      () =>
        setResults((prev) => ({
          ...prev,
          claim: 'Successfully claimed all pending receives!',
        }))
    );
  };

  const handleTryClaimAllLightningReceivesWait = () => {
    runOperation(
      'tryClaimAllLightningReceives (wait)',
      () => NitroArk.tryClaimAllLightningReceives(true),
      'claim',
      () =>
        setResults((prev) => ({
          ...prev,
          claim: 'Successfully claimed all pending receives!',
        }))
    );
  };

  // --- Address Generation ---
  const handleNewAddress = () => {
    runOperation('newAddress', () => NitroArk.newAddress(), 'address');
  };

  const handlePeakAddress = () => {
    runOperation('peakAddress', () => NitroArk.peakAddress(0), 'address');
  };

  const handleGetOnchainAddress = () => {
    runOperation('onchainAddress', () => NitroArk.onchainAddress(), 'address');
  };

  return (
    <ScrollView style={styles.container} showsVerticalScrollIndicator={false}>
      {/* Addresses */}
      <Section title="Receive Addresses">
        <ButtonGrid>
          <CustomButton
            title="New Ark Address"
            onPress={handleNewAddress}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
          <CustomButton
            title="Peak Address"
            onPress={handlePeakAddress}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Onchain Address"
            onPress={handleGetOnchainAddress}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ResultBox result={results.address} error={error.address} />
      </Section>

      {/* Create Invoice */}
      <Section title="Create Invoice">
        <InputField
          label="Amount (sats)"
          value={invoiceAmount}
          onChangeText={setInvoiceAmount}
          placeholder="e.g., 1000"
          keyboardType="numeric"
        />
        <ButtonGrid>
          <CustomButton
            title="Create Invoice"
            onPress={handleCreateInvoice}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
        </ButtonGrid>

        {lastInvoice && (
          <View style={styles.invoiceBox}>
            <Text style={styles.invoiceLabel}>Last Created Invoice:</Text>
            <Text style={styles.invoiceText} selectable>
              {lastInvoice.invoice}
            </Text>
            <Text style={styles.invoiceLabel}>Payment Hash:</Text>
            <Text style={styles.invoiceHashText} selectable>
              {lastInvoice.paymentHash}
            </Text>
          </View>
        )}

        <ResultBox result={results.invoice} error={error.invoice} />
      </Section>

      {/* Check Receive Status */}
      <Section title="Check Receive Status">
        <InputField
          label="Payment Hash"
          value={paymentHash}
          onChangeText={setPaymentHash}
          placeholder="Enter payment hash"
        />
        <ButtonGrid>
          <CustomButton
            title="Get Receive Status"
            onPress={handleLightningReceiveStatus}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ResultBox result={results.status} error={error.status} />
      </Section>

      {/* Claim Payments */}
      <Section title="Claim Payments">
        <InputField
          label="Claim Token (optional)"
          value={claimToken}
          onChangeText={setClaimToken}
          placeholder="Optional token for claiming without VTXOs"
        />
        <ButtonGrid>
          <CustomButton
            title="Claim"
            onPress={handleTryClaimLightningReceive}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
          <CustomButton
            title="Claim (Wait)"
            onPress={handleTryClaimLightningReceiveWait}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Claim All"
            onPress={handleTryClaimAllLightningReceives}
            disabled={walletOpsDisabled}
            color={COLORS.warning}
          />
          <CustomButton
            title="Claim All (Wait)"
            onPress={handleTryClaimAllLightningReceivesWait}
            disabled={walletOpsDisabled}
            color={COLORS.warning}
          />
        </ButtonGrid>
        <ResultBox result={results.claim} error={error.claim} />
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
  invoiceBox: {
    marginTop: 12,
    padding: 12,
    backgroundColor: COLORS.surfaceLight,
    borderRadius: 8,
  },
  invoiceLabel: {
    fontSize: 12,
    color: COLORS.textSecondary,
    marginBottom: 4,
    fontWeight: '600',
  },
  invoiceText: {
    fontSize: 11,
    color: COLORS.text,
    fontFamily: 'monospace',
    marginBottom: 12,
  },
  invoiceHashText: {
    fontSize: 11,
    color: COLORS.primary,
    fontFamily: 'monospace',
  },
  bottomPadding: {
    height: 40,
  },
});
