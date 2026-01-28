"use client";

import { useEffect } from "react";
import { X } from "lucide-react";
import Image from "next/image";

interface SharePopupProps {
  isOpen: boolean;
  onClose: () => void;
  amountRaised: number;
  targetAmount: number;
}

export function SharePopup({
  isOpen,
  onClose,
  amountRaised,
  targetAmount,
}: SharePopupProps) {
  const remainingAmount = targetAmount - amountRaised;

  useEffect(() => {
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

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-[#070A11]"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="share-popup-title"
    >
      {/* Main popup container */}
      <div
        className="relative w-full max-w-[820px] aspect-square"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Gradient blur effects */}
        <div className="absolute w-[387px] h-[387px] left-0 top-0 bg-[#001D62] rounded-full blur-[300px] opacity-80" />
        <div className="absolute w-[387px] h-[387px] right-0 bottom-0 bg-[#DD4FC5] rounded-full blur-[300px] opacity-80" />

        {/* Content card */}
        <div className="relative w-full h-full bg-[#070A11]/90 border border-[#232542] rounded-lg backdrop-blur-sm overflow-hidden">
          {/* Close button */}
          <button
            type="button"
            onClick={onClose}
            className="absolute top-4 right-4 z-20 text-gray-400 hover:text-white transition-colors p-2 rounded-full hover:bg-white/10"
            aria-label="Close popup"
          >
            <X size={24} />
          </button>

          {/* Logo/Brand */}
          <div className="absolute left-[50px] top-[50px] flex items-center gap-3 px-1 py-1 pr-3 border border-[#232542] rounded-full backdrop-blur-md">
            <Image
              src="/navLogo.svg"
              alt="Paymesh Logo"
              width={48}
              height={48}
              className="rounded-full"
            />
            <span className="font-anton text-[#DFDFE0] text-[28px] leading-[48px] uppercase pr-2">
              PAYMESH
            </span>
          </div>

          {/* 3D Coin Stack - centered background */}
          <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[70%] max-w-[610px] aspect-square opacity-40 pointer-events-none">
            <Image
              src="/3d-render--dollar-coins.svg"
              alt=""
              fill
              className="object-contain"
              priority
            />
          </div>

          {/* Stats section */}
          <div className="absolute left-1/2 -translate-x-1/2 top-[200px] w-[90%] max-w-[700px] flex justify-between items-start gap-4 px-4">
            <div className="flex flex-col items-start flex-1">
              <p className="text-[#8398AD] text-sm md:text-base font-bold mb-1">
                Amount Raised
              </p>
              <p className="font-anton text-[#BCC0C5] text-2xl md:text-3xl">
                {amountRaised} USDT
              </p>
            </div>
            <div className="flex flex-col items-start flex-1">
              <p className="text-[#8398AD] text-sm md:text-base font-bold mb-1">
                Remaining
              </p>
              <p className="font-anton text-[#BCC0C5] text-2xl md:text-3xl">
                {remainingAmount} USDT
              </p>
            </div>
            <div className="flex flex-col items-start flex-1">
              <p className="text-[#8398AD] text-sm md:text-base font-bold mb-1">
                Target Amount
              </p>
              <p className="font-anton text-2xl md:text-3xl bg-linear-to-r from-[#92FFB0] to-[#DC92FF] bg-clip-text text-transparent">
                {targetAmount} USDT
              </p>
            </div>
          </div>

          {/* Main headline */}
          <div className="absolute left-1/2 -translate-x-1/2 top-[458px] w-[90%] max-w-[715px] px-4">
            <h3
              id="share-popup-title"
              className="font-anton text-[#DFDFE0] text-5xl md:text-6xl lg:text-[80px] lg:leading-[80px] leading-tight text-center uppercase"
            >
              A LITTLE SUPPORT CAN MAKE A BIG DIFFERENCE.
            </h3>
          </div>
        </div>
      </div>
    </div>
  );
}
