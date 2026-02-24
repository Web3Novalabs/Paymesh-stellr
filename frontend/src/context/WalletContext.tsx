"use client";

import {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  ReactNode,
} from "react";
import {
  connect,
  disconnect,
  getPublicKey,
  connectToWallet,
} from "@/hooks/stellar-wallets-kit";

interface WalletContextType {
  publicKey: string | null;
  isConnected: boolean;
  isLoading: boolean;
  connectWallet: (walletId?: string) => Promise<void>;
  disconnectWallet: () => Promise<void>;
}

const WalletContext = createContext<WalletContextType | null>(null);

export function WalletProvider({ children }: { children: ReactNode }) {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    (async () => {
      const key = await getPublicKey();
      setPublicKey(key);
      setIsLoading(false);
    })();
  }, []);

  const connectWallet = useCallback(async (walletId?: string) => {
    if (walletId) {
      await connectToWallet(walletId, async () => {
        const key = await getPublicKey();
        setPublicKey(key);
      });
    } else {
      await connect(async () => {
        const key = await getPublicKey();
        setPublicKey(key);
      });
    }
  }, []);

  const disconnectWallet = useCallback(async () => {
    await disconnect(async () => {
      setPublicKey(null);
    });
  }, []);

  return (
    <WalletContext.Provider
      value={{
        publicKey,
        isConnected: !!publicKey,
        isLoading,
        connectWallet,
        disconnectWallet,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
}

export function useWallet(): WalletContextType {
  const context = useContext(WalletContext);
  if (!context) {
    throw new Error("useWallet must be used within a WalletProvider");
  }
  return context;
}
