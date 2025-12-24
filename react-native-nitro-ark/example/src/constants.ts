import { Platform } from 'react-native';
import RNFSTurbo from 'react-native-fs-turbo';
import type { BarkCreateOpts } from 'react-native-nitro-ark';

export const ARK_DATA_PATH = `${RNFSTurbo.DocumentDirectoryPath}/bark_data`;
export const MNEMONIC_STORAGE_KEY = 'NITRO_ARK_MNEMONIC';

export const COLORS = {
  primary: '#6366F1',
  primaryDark: '#4F46E5',
  secondary: '#8B5CF6',
  success: '#10B981',
  warning: '#F59E0B',
  danger: '#EF4444',
  background: '#0F172A',
  surface: '#1E293B',
  surfaceLight: '#334155',
  border: '#475569',
  text: '#F8FAFC',
  textSecondary: '#94A3B8',
  textMuted: '#64748B',
};

export const getWalletConfig = (mnemonic: string): BarkCreateOpts => {
  const opts: BarkCreateOpts = {
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

  return opts;
};

// Signet config (commented out for reference)
// export const getSignetWalletConfig = (mnemonic: string): BarkCreateOpts => ({
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
// });

export const formatSats = (sats: number | undefined): string => {
  if (sats === undefined || isNaN(sats)) {
    return 'N/A';
  }
  return `${sats.toLocaleString()} sats`;
};
