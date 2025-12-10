import type {
  BarkArkInfo,
  OnchainBalanceResult,
  OffchainBalanceResult,
} from 'react-native-nitro-ark';

export interface AppState {
  mnemonic: string | undefined;
  arkInfo: BarkArkInfo | undefined;
  onchainBalance: OnchainBalanceResult | undefined;
  offchainBalance: OffchainBalanceResult | undefined;
  results: { [key: string]: string };
  error: { [key: string]: string };
  isLoading: boolean;
}

export interface InputState {
  onchainDestinationAddress: string;
  onchainAmountSat: string;
  arkDestinationAddress: string;
  arkAmountSat: string;
  arkComment: string;
  vtxoIdsInput: string;
  optionalAddress: string;
  invoiceAmount: string;
  invoiceToClaim: string;
  messageToSign: string;
  signature: string;
  publicKeyForVerification: string;
  arkoorAddressToValidate: string;
  paymentHash: string;
}

export type TabName = 'wallet' | 'send' | 'receive';

export interface TabProps {
  mnemonic: string | undefined;
  setMnemonic: (mnemonic: string | undefined) => void;
  arkInfo: BarkArkInfo | undefined;
  setArkInfo: (info: BarkArkInfo | undefined) => void;
  onchainBalance: OnchainBalanceResult | undefined;
  setOnchainBalance: (balance: OnchainBalanceResult | undefined) => void;
  offchainBalance: OffchainBalanceResult | undefined;
  setOffchainBalance: (balance: OffchainBalanceResult | undefined) => void;
  results: { [key: string]: string };
  setResults: React.Dispatch<React.SetStateAction<{ [key: string]: string }>>;
  error: { [key: string]: string };
  setError: React.Dispatch<React.SetStateAction<{ [key: string]: string }>>;
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  isWalletLoaded: boolean;
  setIsWalletLoaded: (loaded: boolean) => void;
  runOperation: (
    operationName: string,
    operationFn: () => Promise<any>,
    section: string,
    updateStateFn?: (result: any) => void
  ) => Promise<void>;
}
