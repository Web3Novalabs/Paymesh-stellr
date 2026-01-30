"use client";

import { connect, disconnect, getPublicKey } from "@/hooks/stellar-wallets-kit";
import { useEffect, useState } from "react";

export default function ConnectWalletButton() {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function showConnected() {
    const key = await getPublicKey();
    if (key) {
      setPublicKey(key);
    } else {
      setPublicKey(null);
    }
    setLoading(false);
  }

  async function showDisconnected() {
    setPublicKey(null);
    setLoading(false);
  }

  useEffect(() => {
    (async () => {
      const key = await getPublicKey();
      if (key) {
        setPublicKey(key);
      }
      setLoading(false);
    })();
  }, []);

  return (
    <div id="connect-wrap" className="wrap" aria-live="polite">
      {!loading && publicKey && (
        <>
          <div className="ellipsis" title={publicKey}>
            Signed in as {publicKey}
          </div>
          <button onClick={() => disconnect(showDisconnected)}>
            Disconnect
          </button>
        </>
      )}

      {!loading && !publicKey && (
        <>
          <button
            onClick={() => connect(showConnected)}
            className="bg-[#5B63D6] hover:bg-[#4A51C9] text-white px-3 lg:px-6 py-[11px] lg:py-[15px] rounded-full text-xs lg:text-sm/[100%] font-black tracking-[0] uppercase transition-colors shadow-lg shadow-indigo-500/20"
          >
            CONNECT WALLET
          </button>
        </>
      )}
    </div>
  );
}
