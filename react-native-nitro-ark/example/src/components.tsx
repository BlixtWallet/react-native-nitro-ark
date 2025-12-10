import React from 'react';
import {
  Text,
  View,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { COLORS } from './constants';

interface CustomButtonProps {
  title: string;
  onPress: () => void;
  disabled?: boolean;
  color?: string;
  small?: boolean;
}

export const CustomButton = ({
  title,
  onPress,
  disabled,
  color,
  small,
}: CustomButtonProps) => (
  <TouchableOpacity
    style={[
      styles.customButton,
      { backgroundColor: color || COLORS.primary },
      disabled && styles.customButtonDisabled,
      small && styles.customButtonSmall,
    ]}
    onPress={onPress}
    disabled={disabled}
  >
    <Text
      style={[styles.customButtonText, small && styles.customButtonTextSmall]}
    >
      {title}
    </Text>
  </TouchableOpacity>
);

interface InputFieldProps {
  label: string;
  value: string;
  onChangeText: (text: string) => void;
  placeholder?: string;
  keyboardType?: 'default' | 'numeric' | 'email-address';
  multiline?: boolean;
}

export const InputField = ({
  label,
  value,
  onChangeText,
  placeholder,
  keyboardType = 'default',
  multiline = false,
}: InputFieldProps) => (
  <View style={styles.inputContainer}>
    <Text style={styles.inputLabel}>{label}</Text>
    <TextInput
      style={[styles.input, multiline && styles.inputMultiline]}
      value={value}
      onChangeText={onChangeText}
      placeholder={placeholder}
      placeholderTextColor={COLORS.textMuted}
      keyboardType={keyboardType}
      autoCapitalize="none"
      multiline={multiline}
    />
  </View>
);

interface ResultBoxProps {
  result?: string;
  error?: string;
}

export const ResultBox = ({ result, error }: ResultBoxProps) => {
  if (!result && !error) return null;

  return (
    <View style={styles.resultContainer}>
      {error ? (
        <View style={styles.errorBox}>
          <Text style={styles.errorHeader}>Error</Text>
          <Text style={styles.errorText}>{error}</Text>
        </View>
      ) : result ? (
        <View style={styles.resultBox}>
          <Text style={styles.resultHeader}>Result</Text>
          <Text style={styles.resultText}>{result}</Text>
        </View>
      ) : null}
    </View>
  );
};

interface SectionProps {
  title: string;
  children: React.ReactNode;
}

export const Section = ({ title, children }: SectionProps) => (
  <View style={styles.section}>
    <Text style={styles.sectionTitle}>{title}</Text>
    {children}
  </View>
);

interface ButtonGridProps {
  children: React.ReactNode;
}

export const ButtonGrid = ({ children }: ButtonGridProps) => (
  <View style={styles.buttonGrid}>{children}</View>
);

interface LoadingOverlayProps {
  visible: boolean;
}

export const LoadingOverlay = ({ visible }: LoadingOverlayProps) => {
  if (!visible) return null;
  return (
    <View style={styles.loadingOverlay}>
      <View style={styles.loadingBox}>
        <ActivityIndicator size="large" color={COLORS.primary} />
        <Text style={styles.loadingText}>Processing...</Text>
      </View>
    </View>
  );
};

interface BalanceCardProps {
  title: string;
  balances: { label: string; value: string }[];
}

export const BalanceCard = ({ title, balances }: BalanceCardProps) => (
  <View style={styles.balanceCard}>
    <Text style={styles.balanceCardTitle}>{title}</Text>
    {balances.map((item, index) => (
      <View key={index} style={styles.balanceRow}>
        <Text style={styles.balanceLabel}>{item.label}</Text>
        <Text style={styles.balanceValue}>{item.value}</Text>
      </View>
    ))}
  </View>
);

interface InfoRowProps {
  label: string;
  value: string;
}

export const InfoRow = ({ label, value }: InfoRowProps) => (
  <View style={styles.infoRow}>
    <Text style={styles.infoLabel}>{label}</Text>
    <Text style={styles.infoValue} numberOfLines={1} ellipsizeMode="middle">
      {value}
    </Text>
  </View>
);

const styles = StyleSheet.create({
  customButton: {
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 44,
    marginVertical: 4,
    flex: 1,
    marginHorizontal: 4,
  },
  customButtonSmall: {
    paddingVertical: 8,
    paddingHorizontal: 12,
    minHeight: 36,
  },
  customButtonDisabled: {
    opacity: 0.5,
  },
  customButtonText: {
    color: COLORS.text,
    fontSize: 14,
    fontWeight: '600',
    textAlign: 'center',
  },
  customButtonTextSmall: {
    fontSize: 12,
  },
  inputContainer: {
    marginVertical: 8,
  },
  inputLabel: {
    fontSize: 13,
    fontWeight: '500',
    marginBottom: 6,
    color: COLORS.textSecondary,
  },
  input: {
    borderWidth: 1,
    borderColor: COLORS.border,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    backgroundColor: COLORS.surface,
    color: COLORS.text,
  },
  inputMultiline: {
    minHeight: 80,
    textAlignVertical: 'top',
  },
  resultContainer: {
    marginTop: 12,
  },
  resultBox: {
    padding: 12,
    backgroundColor: COLORS.surface,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: COLORS.success,
  },
  resultHeader: {
    fontWeight: '600',
    marginBottom: 8,
    color: COLORS.success,
    fontSize: 13,
  },
  resultText: {
    fontSize: 12,
    color: COLORS.text,
    fontFamily: 'monospace',
  },
  errorBox: {
    padding: 12,
    backgroundColor: 'rgba(239, 68, 68, 0.1)',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: COLORS.danger,
  },
  errorHeader: {
    fontWeight: '600',
    marginBottom: 8,
    color: COLORS.danger,
    fontSize: 13,
  },
  errorText: {
    fontSize: 12,
    color: COLORS.danger,
  },
  section: {
    marginBottom: 20,
    padding: 16,
    backgroundColor: COLORS.surface,
    borderRadius: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '700',
    marginBottom: 12,
    color: COLORS.text,
  },
  buttonGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginHorizontal: -4,
    marginTop: 8,
  },
  loadingOverlay: {
    position: 'absolute',
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(15, 23, 42, 0.8)',
    zIndex: 1000,
  },
  loadingBox: {
    backgroundColor: COLORS.surface,
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 12,
    color: COLORS.text,
    fontSize: 14,
  },
  balanceCard: {
    backgroundColor: COLORS.surfaceLight,
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
  },
  balanceCardTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: COLORS.text,
    marginBottom: 8,
  },
  balanceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 4,
  },
  balanceLabel: {
    fontSize: 13,
    color: COLORS.textSecondary,
  },
  balanceValue: {
    fontSize: 13,
    color: COLORS.text,
    fontWeight: '500',
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 6,
    borderBottomWidth: 1,
    borderBottomColor: COLORS.border,
  },
  infoLabel: {
    fontSize: 13,
    color: COLORS.textSecondary,
    flex: 1,
  },
  infoValue: {
    fontSize: 13,
    color: COLORS.text,
    flex: 2,
    textAlign: 'right',
  },
});
