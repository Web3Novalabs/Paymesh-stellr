"use client";

import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { X } from "lucide-react";
import Image from "next/image";
import { motion, AnimatePresence } from "framer-motion";
import toast from "react-hot-toast";
import {
  ALBEDO_ID,
  FREIGHTER_ID,
  LOBSTR_ID,
} from "@creit.tech/stellar-wallets-kit";
import { useWallet } from "@/context/WalletContext";

interface StellarWalletModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConnected: (publicKey: string) => void;
}

const WALLETS = [
  {
    id: FREIGHTER_ID,
    name: "FREIGHTER WALLET",
    icon: "/stllr-wallets/freighter.png",
    color: "#1B4791",
  },
  {
    id: ALBEDO_ID,
    name: "ALBEDO WALLET",
    icon: "/stllr-wallets/albedo.svg",
    color: "#422896",
  },
  {
    id: LOBSTR_ID,
    name: "LOBSTR WALLET",
    icon: "/stllr-wallets/lobstr.svg",
    color: "#DF643B",
  },
];

export function StellarWalletModal({
  isOpen,
  onClose,
  onConnected,
}: StellarWalletModalProps) {
  const [connecting, setConnecting] = useState<string | null>(null);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    if (isOpen) {
      document.body.style.overflow = "hidden";
      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === "Escape") onClose();
      };
      window.addEventListener("keydown", handleEscape);
      return () => {
        document.body.style.overflow = "unset";
        window.removeEventListener("keydown", handleEscape);
      };
    }
  }, [isOpen, onClose]);

  const { connectWallet } = useWallet();

  const handleConnect = async (walletId: string) => {
    if (connecting) return;

    setConnecting(walletId);

    // Create a timeout to prevent hanging
    const timeoutPromise = new Promise((_, reject) => {
      setTimeout(() => reject(new Error("Connection timeout")), 15000);
    });

    try {
      await Promise.race([connectWallet(walletId), timeoutPromise]);
      toast.success("Wallet connected!");
      onClose();
    } catch (error: unknown) {
      console.error("Failed to connect wallet:", error);
      let message = "Failed to connect";

      if (error instanceof Error) {
        message = error.message;
      } else if (typeof error === "string") {
        message = error;
      } else if (
        typeof error === "object" &&
        error !== null &&
        "message" in error
      ) {
        message = String((error as Record<string, unknown>).message);
      }

      if (message.includes("rejected")) {
        toast.error("Connection rejected by user");
      } else if (message.includes("timeout")) {
        toast.error("Connection timed out. Please try again.");
      } else if (
        message.includes("not connected") ||
        message.includes("not found")
      ) {
        toast.error("Wallet extension not detected or not active");
      } else {
        toast.error(`Connection failed: ${message}`);
      }
    } finally {
      setConnecting(null);
    }
  };

  if (!mounted) return null;

  return createPortal(
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4 sm:p-6">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-black/60 backdrop-blur-md"
            onClick={onClose}
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.9, y: 30 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: 30 }}
            className="relative w-full max-w-[420px] bg-[#0D0D10] border border-[#232542] rounded-[16px] overflow-hidden shadow-[0_20px_50px_rgba(0,0,0,0.5)]"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-center justify-between px-6 pt-6 pb-2">
              <h2 className="text-[22px] font-anton text-[#DFDFE0] uppercase tracking-wide">
                CONNECT WALLET
              </h2>
              <button
                onClick={onClose}
                className="text-gray-400 hover:text-white transition-colors p-1 rounded-full hover:bg-white/5"
              >
                <X size={24} />
              </button>
            </div>

            {/* Wallet Options */}
            <div className="p-6 space-y-3">
              {WALLETS.map((wallet) => (
                <button
                  key={wallet.id}
                  onClick={() => handleConnect(wallet.id)}
                  disabled={!!connecting}
                  className="group relative w-full flex items-center justify-between h-[84px] p-[2px] rounded-[12px] overflow-hidden transition-all duration-300 hover:scale-[1.01] active:scale-[0.99] disabled:opacity-50 disabled:scale-100"
                >
                  {/* Animated Border/Background */}
                  <div
                    className="absolute inset-0 opacity-80 group-hover:opacity-100 transition-opacity"
                    style={{
                      background: `linear-gradient(135deg, ${wallet.color} 0%, rgba(20, 20, 25, 0.9) 100%)`,
                    }}
                  />

                  <div className="relative z-10 w-full h-full flex items-center justify-between px-4 bg-[#0D0D10]/40 group-hover:bg-transparent transition-colors rounded-[10px]">
                    <div className="flex items-center gap-4">
                      <div className="relative w-14 h-14 flex items-center justify-center rounded-xl bg-white/10 group-hover:bg-white/20 transition-colors shadow-inner overflow-hidden border border-white/10">
                        <div className=" backdrop-blur-sm" />
                        <Image
                          src={wallet.icon}
                          alt={wallet.name}
                          width={60}
                          height={40}
                          className="relative z-10 object-contain p-1"
                        />
                      </div>
                      <span className="font-anton text-white text-[15px] tracking-wide uppercase">
                        {connecting === wallet.id
                          ? "CONNECTING..."
                          : wallet.name}
                      </span>
                    </div>

                    <div className="flex items-center justify-center w-[64px] h-[36px] border border-white/20 rounded-full bg-white/5 group-hover:bg-white/20 transition-all">
                      <span className="text-[11px] font-black text-white uppercase tracking-tight">
                        Use
                      </span>
                    </div>
                  </div>
                </button>
              ))}
            </div>

            {/* Decoration */}
            <div className="h-1 w-full bg-gradient-to-r from-transparent via-[#5B63D6] to-transparent opacity-20" />
          </motion.div>
        </div>
      )}
    </AnimatePresence>,
    document.body,
  );
}
