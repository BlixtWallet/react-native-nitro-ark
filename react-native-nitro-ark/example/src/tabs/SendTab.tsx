import { useState } from 'react';
import { View, ScrollView, StyleSheet } from 'react-native';
import * as NitroArk from 'react-native-nitro-ark';
import type { BarkSendManyOutput } from 'react-native-nitro-ark';

import {
  CustomButton,
  InputField,
  ResultBox,
  Section,
  ButtonGrid,
} from '../components';
import { COLORS } from '../constants';
import type { TabProps } from '../types';

export const SendTab = ({
  mnemonic,
  results,
  setResults,
  error,
  setError,
  isLoading,
  runOperation,
}: TabProps) => {
  // Onchain inputs
  const [onchainDestination, setOnchainDestination] = useState('');
  const [onchainAmount, setOnchainAmount] = useState('');

  // Ark inputs
  const [arkDestination, setArkDestination] = useState('');
  const [arkAmount, setArkAmount] = useState('');
  const [arkComment, setArkComment] = useState('');
  const [arkoorAddressToValidate, setArkoorAddressToValidate] = useState('');

  // Lightning payment check
  const [paymentHash, setPaymentHash] = useState('');

  // Offboard inputs
  const [vtxoIdsInput, setVtxoIdsInput] = useState('');
  const [offboardDestination, setOffboardDestination] = useState('');

  const canUseWallet = !!mnemonic;
  const walletOpsDisabled = isLoading || !canUseWallet;

  // --- Onchain Operations ---
  const handleSendOnchain = () => {
    if (!onchainDestination || !onchainAmount) {
      setError((prev) => ({
        ...prev,
        onchain: 'Destination and amount required',
      }));
      return;
    }
    const amount = parseInt(onchainAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, onchain: 'Invalid amount' }));
      return;
    }
    runOperation(
      'onchainSend',
      () => NitroArk.onchainSend(onchainDestination, amount),
      'onchain'
    );
  };

  const handleDrainOnchain = () => {
    if (!onchainDestination) {
      setError((prev) => ({ ...prev, onchain: 'Destination required' }));
      return;
    }
    runOperation(
      'onchainDrain',
      () => NitroArk.onchainDrain(onchainDestination),
      'onchain'
    );
  };

  const handleSendManyOnchain = () => {
    if (!onchainDestination || !onchainAmount) {
      setError((prev) => ({
        ...prev,
        onchain: 'At least one destination and amount required',
      }));
      return;
    }
    const amount = parseInt(onchainAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, onchain: 'Invalid amount' }));
      return;
    }
    const outputs: BarkSendManyOutput[] = [
      { destination: onchainDestination, amountSat: amount },
    ];
    runOperation(
      'onchainSendMany',
      () => NitroArk.onchainSendMany(outputs),
      'onchain'
    );
  };

  // --- Board Operations ---
  const handleBoardAmount = () => {
    if (!arkAmount) {
      setError((prev) => ({ ...prev, ark: 'Amount required' }));
      return;
    }
    const amount = parseInt(arkAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount' }));
      return;
    }
    runOperation('boardAmount', () => NitroArk.boardAmount(amount), 'ark');
  };

  const handleBoardAll = () => {
    runOperation('boardAll', () => NitroArk.boardAll(), 'ark');
  };

  // --- Ark Payments ---
  const handleSendArkoorPayment = () => {
    if (!arkDestination || !arkAmount) {
      setError((prev) => ({ ...prev, ark: 'Destination and amount required' }));
      return;
    }
    const amount = parseInt(arkAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount' }));
      return;
    }
    runOperation(
      'sendArkoorPayment',
      () => NitroArk.sendArkoorPayment(arkDestination, amount),
      'ark'
    );
  };

  const handleValidateArkoorAddress = () => {
    if (!arkoorAddressToValidate) {
      setError((prev) => ({ ...prev, ark: 'Address required' }));
      return;
    }
    runOperation(
      'validateArkoorAddress',
      () => NitroArk.validateArkoorAddress(arkoorAddressToValidate),
      'ark',
      () => setResults((prev) => ({ ...prev, ark: 'Address is valid!' }))
    );
  };

  // --- Lightning Payments ---
  const handlePayLightningInvoice = () => {
    if (!arkDestination) {
      setError((prev) => ({ ...prev, lightning: 'Invoice required' }));
      return;
    }
    const amount = parseInt(arkAmount, 10) || undefined;
    runOperation(
      'payLightningInvoice',
      () => NitroArk.payLightningInvoice(arkDestination, amount),
      'lightning'
    );
  };

  const handlePayLightningOffer = () => {
    if (!arkDestination) {
      setError((prev) => ({ ...prev, lightning: 'Offer required' }));
      return;
    }
    const amount = parseInt(arkAmount, 10) || undefined;
    runOperation(
      'payLightningOffer',
      () => NitroArk.payLightningOffer(arkDestination, amount),
      'lightning'
    );
  };

  const handlePayLightningAddress = () => {
    if (!arkDestination || !arkAmount) {
      setError((prev) => ({
        ...prev,
        lightning: 'Lightning address and amount required',
      }));
      return;
    }
    const amount = parseInt(arkAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, lightning: 'Invalid amount' }));
      return;
    }
    runOperation(
      'payLightningAddress',
      () => NitroArk.payLightningAddress(arkDestination, amount, arkComment),
      'lightning'
    );
  };

  // --- Check Lightning Payment Status ---
  const handleCheckLightningPayment = () => {
    if (!paymentHash) {
      setError((prev) => ({ ...prev, lnstatus: 'Payment hash required' }));
      return;
    }
    runOperation(
      'checkLightningPayment',
      () => NitroArk.checkLightningPayment(paymentHash, false),
      'lnstatus',
      (preimage) => {
        if (preimage) {
          setResults((prev) => ({
            ...prev,
            lnstatus: `Payment confirmed!\n\nPreimage: ${preimage}`,
          }));
        } else {
          setResults((prev) => ({
            ...prev,
            lnstatus: 'Payment not yet confirmed',
          }));
        }
      }
    );
  };

  const handleCheckLightningPaymentWait = () => {
    if (!paymentHash) {
      setError((prev) => ({ ...prev, lnstatus: 'Payment hash required' }));
      return;
    }
    runOperation(
      'checkLightningPayment (wait)',
      () => NitroArk.checkLightningPayment(paymentHash, true),
      'lnstatus',
      (preimage) => {
        if (preimage) {
          setResults((prev) => ({
            ...prev,
            lnstatus: `Payment confirmed!\n\nPreimage: ${preimage}`,
          }));
        } else {
          setResults((prev) => ({
            ...prev,
            lnstatus: 'Timed out waiting for confirmation',
          }));
        }
      }
    );
  };

  const handleSendRoundOnchainPayment = () => {
    if (!arkDestination || !arkAmount) {
      setError((prev) => ({
        ...prev,
        ark: 'Destination and amount required',
      }));
      return;
    }
    const amount = parseInt(arkAmount, 10);
    if (isNaN(amount) || amount <= 0) {
      setError((prev) => ({ ...prev, ark: 'Invalid amount' }));
      return;
    }
    runOperation(
      'sendRoundOnchainPayment',
      () => NitroArk.sendRoundOnchainPayment(arkDestination, amount),
      'ark'
    );
  };

  // --- Offboarding ---
  const handleOffboardSpecific = () => {
    if (!vtxoIdsInput || !offboardDestination) {
      setError((prev) => ({
        ...prev,
        offboard: 'VTXO IDs and destination required',
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
        offboard: 'At least one VTXO ID required',
      }));
      return;
    }
    runOperation(
      'offboardSpecific',
      () => NitroArk.offboardSpecific(ids, offboardDestination),
      'offboard'
    );
  };

  const handleOffboardAll = () => {
    if (!offboardDestination) {
      setError((prev) => ({ ...prev, offboard: 'Destination required' }));
      return;
    }
    runOperation(
      'offboardAll',
      () => NitroArk.offboardAll(offboardDestination),
      'offboard'
    );
  };

  return (
    <ScrollView style={styles.container} showsVerticalScrollIndicator={false}>
      {/* Onchain Send */}
      <Section title="Onchain Send">
        <InputField
          label="Destination Address"
          value={onchainDestination}
          onChangeText={setOnchainDestination}
          placeholder="bc1q... or similar"
        />
        <InputField
          label="Amount (sats)"
          value={onchainAmount}
          onChangeText={setOnchainAmount}
          placeholder="e.g., 10000"
          keyboardType="numeric"
        />
        <ButtonGrid>
          <CustomButton
            title="Send"
            onPress={handleSendOnchain}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
          <CustomButton
            title="Drain"
            onPress={handleDrainOnchain}
            disabled={walletOpsDisabled}
            color={COLORS.warning}
          />
          <CustomButton
            title="Send Many"
            onPress={handleSendManyOnchain}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ResultBox result={results.onchain} error={error.onchain} />
      </Section>

      {/* Board (Onchain → Ark) */}
      <Section title="Board (Onchain → Ark)">
        <InputField
          label="Amount to Board (sats)"
          value={arkAmount}
          onChangeText={setArkAmount}
          placeholder="e.g., 50000"
          keyboardType="numeric"
        />
        <ButtonGrid>
          <CustomButton
            title="Board Amount"
            onPress={handleBoardAmount}
            disabled={walletOpsDisabled}
            color={COLORS.success}
          />
          <CustomButton
            title="Board All"
            onPress={handleBoardAll}
            disabled={walletOpsDisabled}
            color={COLORS.warning}
          />
        </ButtonGrid>
        <ResultBox result={results.board} error={error.board} />
      </Section>

      {/* Ark Payments */}
      <Section title="Ark Payments">
        <InputField
          label="Destination (Arkoor address / Invoice / Offer / LN Address)"
          value={arkDestination}
          onChangeText={setArkDestination}
          placeholder="ark1..., lnbc..., lno..., user@domain"
          multiline
        />
        <InputField
          label="Amount (sats) - Optional for invoices"
          value={arkAmount}
          onChangeText={setArkAmount}
          placeholder="e.g., 1000"
          keyboardType="numeric"
        />
        <InputField
          label="Comment (for LN Address)"
          value={arkComment}
          onChangeText={setArkComment}
          placeholder="Optional comment"
        />
        <ButtonGrid>
          <CustomButton
            title="Send Arkoor"
            onPress={handleSendArkoorPayment}
            disabled={walletOpsDisabled}
            color={COLORS.primary}
          />
          <CustomButton
            title="Round Onchain"
            onPress={handleSendRoundOnchainPayment}
            disabled={walletOpsDisabled}
          />
        </ButtonGrid>
        <ResultBox result={results.ark} error={error.ark} />
      </Section>

      {/* Lightning Payments */}
      <Section title="Lightning Payments">
        <ButtonGrid>
          <CustomButton
            title="Pay Invoice (Bolt11)"
            onPress={handlePayLightningInvoice}
            disabled={walletOpsDisabled}
            color={COLORS.secondary}
          />
          <CustomButton
            title="Pay Offer (Bolt12)"
            onPress={handlePayLightningOffer}
            disabled={walletOpsDisabled}
            color={COLORS.secondary}
          />
        </ButtonGrid>
        <ButtonGrid>
          <CustomButton
            title="Pay LN Address"
            onPress={handlePayLightningAddress}
            disabled={walletOpsDisabled}
            color={COLORS.secondary}
          />
        </ButtonGrid>
        <ResultBox result={results.lightning} error={error.lightning} />
      </Section>

      {/* Check Lightning Payment Status */}
      <Section title="Check Lightning Send Status">
        <InputField
          label="Payment Hash"
          value={paymentHash}
          onChangeText={setPaymentHash}
          placeholder="Enter payment hash to check"
        />
        <ButtonGrid>
          <CustomButton
            title="Check Status"
            onPress={handleCheckLightningPayment}
            disabled={walletOpsDisabled}
          />
          <CustomButton
            title="Check (Wait)"
            onPress={handleCheckLightningPaymentWait}
            disabled={walletOpsDisabled}
            color={COLORS.secondary}
          />
        </ButtonGrid>
        <ResultBox result={results.lnstatus} error={error.lnstatus} />
      </Section>

      {/* Validate Arkoor Address */}
      <Section title="Validate Address">
        <InputField
          label="Arkoor Address"
          value={arkoorAddressToValidate}
          onChangeText={setArkoorAddressToValidate}
          placeholder="Enter Arkoor address to validate"
        />
        <ButtonGrid>
          <CustomButton
            title="Validate"
            onPress={handleValidateArkoorAddress}
            disabled={isLoading}
          />
        </ButtonGrid>
      </Section>

      {/* Offboarding (Ark → Onchain) */}
      <Section title="Offboard (Ark → Onchain)">
        <InputField
          label="Destination Address"
          value={offboardDestination}
          onChangeText={setOffboardDestination}
          placeholder="bc1q... onchain address"
        />
        <InputField
          label="VTXO IDs (comma-separated)"
          value={vtxoIdsInput}
          onChangeText={setVtxoIdsInput}
          placeholder="vtxo1, vtxo2, ..."
          multiline
        />
        <ButtonGrid>
          <CustomButton
            title="Offboard Specific"
            onPress={handleOffboardSpecific}
            disabled={walletOpsDisabled}
            color={COLORS.warning}
          />
          <CustomButton
            title="Offboard All"
            onPress={handleOffboardAll}
            disabled={walletOpsDisabled}
            color={COLORS.danger}
          />
        </ButtonGrid>
        <ResultBox result={results.offboard} error={error.offboard} />
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
  bottomPadding: {
    height: 40,
  },
});
