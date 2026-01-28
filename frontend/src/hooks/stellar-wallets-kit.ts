import {
  allowAllModules,
  FREIGHTER_ID,
  StellarWalletsKit,
  WalletNetwork,
} from "@creit.tech/stellar-wallets-kit";

const SELECTED_WALLET_ID = "selectedWalletId";

function getSelectedWalletId() {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(SELECTED_WALLET_ID);
}

let kit: StellarWalletsKit | null = null;

function getKit(): StellarWalletsKit | null {
  if (kit) return kit;

  if (typeof window !== "undefined") {
    kit = new StellarWalletsKit({
      modules: allowAllModules(),
      network: WalletNetwork.PUBLIC,
      selectedWalletId: getSelectedWalletId() ?? FREIGHTER_ID,
    });
    return kit;
  }

  return null;
}

export const signTransaction = async (
  ...args: Parameters<StellarWalletsKit["signTransaction"]>
) => {
  const k = getKit();
  if (!k) throw new Error("Wallet kit not initialized");
  return k.signTransaction(...args);
};

export async function getPublicKey() {
  const k = getKit();
  if (!k || !getSelectedWalletId()) return null;

  try {
    const { address } = await k.getAddress();
    return address;
  } catch {
    return null;
  }
}

export async function setWallet(walletId: string) {
  if (typeof window !== "undefined") {
    localStorage.setItem(SELECTED_WALLET_ID, walletId);
  }
  const k = getKit();
  if (k) k.setWallet(walletId);
}

export async function disconnect(callback?: () => Promise<void>) {
  if (typeof window !== "undefined") {
    localStorage.removeItem(SELECTED_WALLET_ID);
  }
  const k = getKit();
  if (k) {
    // kit.disconnect() might not return a promise depending on version,
    // but usually it's void or Promise<void>. safest to await if possible or just call.
    await k.disconnect();
  }
  if (callback) await callback();
}

export async function connect(callback?: () => Promise<void>) {
  const k = getKit();
  if (!k) return;

  await k.openModal({
    onWalletSelected: async (option) => {
      try {
        await setWallet(option.id);
        if (callback) await callback();
      } catch (e) {
        console.error(e);
      }
      return option.id;
    },
  });
}
