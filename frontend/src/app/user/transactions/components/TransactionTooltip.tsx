"use client";

import { useState } from "react";
import Image from "next/image";
// import { Tooltip } from "@/components/ui/tooltip";

export interface TransactionToken {
  name: string;
  ticker: string;
  amount: string | number;
  icon: string;
  value?: string;
}

export function TransactionTooltip({
  tokens,
  children,
  side = "top",
}: {
  tokens: TransactionToken[];
  children: React.ReactNode;
  side?: "top" | "bottom" | "left" | "right";
}) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative inline-block">
      <div
        onMouseEnter={() => setIsOpen(true)}
        onMouseLeave={() => setIsOpen(false)}
        onClick={() => setIsOpen(!isOpen)}
      >
        {children}
      </div>
      {isOpen && (
        <div
          className={`absolute z-50 ${
            side === "bottom" ? "top-full mt-2" : "bottom-full mb-2"
          } left-1/2 -translate-x-1/2`}
        >
          <div className="bg-[#1A1D29] border border-white/10 rounded-xl p-2 sm:p-4 shadow-2xl">
            <div className="flex flex-col gap-2 sm:gap-3 min-w-32 sm:min-w-45">
              <h4 className="text-white font-semibold text-xs sm:text-sm">Paymesh</h4>
              {tokens.map((token, index) => (
                <div key={index} className="flex items-center justify-between gap-2 sm:gap-4">
                  <div className="flex items-center gap-2">
                    <div className="relative w-5 h-5 sm:w-6 sm:h-6 rounded-full overflow-hidden shrink-0">
                      <Image src={token.icon} alt={token.ticker} fill className="object-cover" />
                    </div>
                    <span className="text-[#A0A5BA] text-[10px] sm:text-xs font-medium whitespace-nowrap">
                      {token.ticker}
                    </span>
                  </div>
                  <div className="text-right">
                    <div className="text-white text-xs sm:text-sm font-normal">{token.amount}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

