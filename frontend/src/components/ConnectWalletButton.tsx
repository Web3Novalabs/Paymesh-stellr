"use client";

import { connect, disconnect, getPublicKey } from "@/hooks/stellar-wallets-kit";
import { useEffect, useState, useRef } from "react";
import { MoreVertical, LogOut, User } from "lucide-react";

import { useWallet } from "@/context/WalletContext";
import { StellarWalletModal } from "./StellarWalletModal";

const style = `
  @keyframes gradientShift {
    0% {
      background-position: 0% center;
    }
    100% {
      background-position: 200% center;
    }
  }

  .gradient-border-button {
    position: relative;
    background: transparent;
    border: none;
    padding: 0;
  }

  .gradient-border-button::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: 48px;
    padding: 0.5px;
    background: linear-gradient(90deg, #BCC0C5, #FF929F, #FFAC92, #FFD392, #92FFB0, #92F2FF, #92CAFF, #A192FF, #DC92FF, #BCC0C5);
    background-size: 200% 100%;
    animation: gradientShift 4s linear infinite;
    -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    pointer-events: none;
  }

  .gradient-border-button-inner {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: auto;
    height: auto;
    padding: 14px 24px;
    border-radius: 48px;
    font-size: 0.75rem;
    font-weight: 600;
    color: #ffffff;
    transition: all 0.3s ease;
  }

  @media (min-width: 640px) {
    .gradient-border-button-inner {
      width: 177px;
      height: 56px;
      padding: 0.25rem 1.5rem;
    }
  }
`;

export default function ConnectWalletButton() {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const { isLoading, connectWallet, disconnectWallet } = useWallet();

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
    setDropdownOpen(false);
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

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setDropdownOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const formattedKey = publicKey
    ? `${publicKey.substring(0, 6)}...${publicKey.substring(publicKey.length - 6)}`
    : "";

  return (
    <>
      <style>{style}</style>
      <div
        id="connect-wrap"
        className="relative"
        aria-live="polite"
        ref={dropdownRef}
      >
        {!isLoading && publicKey && (
          <>
            <button
              onClick={() => setDropdownOpen(!dropdownOpen)}
              className="gradient-border-button"
            >
              <div className="gradient-border-button-inner">
                <span className="text-white font-black text-sm">
                  {formattedKey}
                </span>
                <div className="hidden sm:block">
                  <MoreVertical size={16} className="text-white" />
                </div>
              </div>
            </button>

            {dropdownOpen && (
              <div className="absolute right-0 mt-2 w-56 bg-[#0D0D10] border border-[#232542] rounded-xl shadow-2xl py-2 z-50 animate-in fade-in slide-in-from-top-2">
                <div className="px-4 py-3 border-b border-[#232542] mb-2">
                  <p className="text-xs text-gray-400 mb-1">Connected Wallet</p>
                  <p
                    className="text-sm font-medium text-white truncate"
                    title={publicKey}
                  >
                    {formattedKey}
                  </p>
                </div>
                <button
                  onClick={async () => {
                    await disconnectWallet();
                    setDropdownOpen(false);
                  }}
                  className="w-full flex items-center gap-3 px-4 py-2.5 text-sm text-[#FF4D4D] hover:bg-[#FF4D4D]/10 transition-colors font-medium"
                >
                  <LogOut size={16} />
                  Disconnect Session
                </button>
              </div>
            )}
          </>
        )}

        {!isLoading && !publicKey && (
          <>
            <button
              onClick={() => setModalOpen(true)}
              className="bg-[#5B63D6] hover:bg-[#4A51C9] text-white px-3 lg:px-6 py-[11px] lg:py-[15px] rounded-full text-xs lg:text-sm/[100%] font-black tracking-[0] uppercase transition-colors shadow-lg shadow-indigo-500/20"
            >
              CONNECT WALLET
            </button>
          </>
        )}
      </div>

      <StellarWalletModal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        onConnected={(key) => {
          // Key status is managed by WalletProvider via the connectWallet(id) call inside the modal
        }}
      />
    </>
  );
}
