"use client";

import { connect, disconnect, getPublicKey } from "@/hooks/stellar-wallets-kit";
import { useEffect, useState, useRef } from "react";
import { ChevronDown, LogOut, User } from "lucide-react";

export default function ConnectWalletButton() {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

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
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setDropdownOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const formattedKey = publicKey ? `${publicKey.substring(0, 4)}...${publicKey.substring(publicKey.length - 4)}` : "";

  return (
    <div id="connect-wrap" className="relative" aria-live="polite" ref={dropdownRef}>
      {!loading && publicKey && (
        <>
          <button
            onClick={() => setDropdownOpen(!dropdownOpen)}
            className="flex items-center gap-2 bg-[#0D0D10] border border-[#232542] hover:bg-[#1A1A24] text-white px-3 lg:px-4 py-[8px] lg:py-[10px] rounded-full text-xs lg:text-sm font-semibold transition-colors"
          >
            <div className="w-6 h-6 rounded-full bg-gradient-to-r from-[#5B63D6] to-[#9D4EDD] flex items-center justify-center">
              <User size={14} className="text-white" />
            </div>
            <span className="hidden sm:inline">{formattedKey}</span>
            <ChevronDown size={16} className={`text-gray-400 transition-transform ${dropdownOpen ? "rotate-180" : ""}`} />
          </button>

          {dropdownOpen && (
            <div className="absolute right-0 mt-2 w-56 bg-[#0D0D10] border border-[#232542] rounded-xl shadow-2xl py-2 z-50 animate-in fade-in slide-in-from-top-2">
              <div className="px-4 py-3 border-b border-[#232542] mb-2">
                <p className="text-xs text-gray-400 mb-1">Connected Wallet</p>
                <p className="text-sm font-medium text-white truncate" title={publicKey}>
                  {formattedKey}
                </p>
              </div>
              <button
                onClick={() => disconnect(showDisconnected)}
                className="w-full flex items-center gap-3 px-4 py-2.5 text-sm text-[#FF4D4D] hover:bg-[#FF4D4D]/10 transition-colors font-medium"
              >
                <LogOut size={16} />
                Disconnect Session
              </button>
            </div>
          )}
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
