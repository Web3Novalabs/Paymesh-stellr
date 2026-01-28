"use client";

import Image from "next/image";
import { TransactionTooltip } from "./TransactionTooltip";

export interface TransactionToken {
  name: string;
  ticker: string;
  amount: string | number;
  icon: string;
  value?: string;
}

export interface TransactionData {
  id: string;
  groupName: string;
  groupAddress: string;
  totalAmount: string;
  tokens: TransactionToken[];
  members: number;
  time: string;
  date: string;
}

interface TransactionRowProps {
  transaction: TransactionData;
  isFirstTwoRows?: boolean;
}

export default function TransactionRow({ transaction, isFirstTwoRows = false }: TransactionRowProps) {
  const truncateAddress = (address: string, length: number = 16) => {
    if (address.length <= length) return address;
    return `${address.slice(0, length)}...`;
  };

return (
  <>
    {/* Desktop & Tablet View (768px+) */}
    <tr className="hidden md:table-row border-b border-white/5 hover:bg-white/3 transition-colors h-18">
      <td className="pl-6 lg:pl-10 py-4">
        <div className="text-white font-medium text-sm lg:text-base">{transaction.groupName}</div>
      </td>
      <td className="px-4 lg:px-10 py-4">
        <div className="text-white font-mono text-sm lg:text-base max-w-35 truncate cursor-help" title={transaction.groupAddress}>
          {truncateAddress(transaction.groupAddress)}
        </div>
      </td>
      <td className="px-4 lg:px-10 py-4">
        <div className="text-white font-semibold text-sm lg:text-base">{transaction.totalAmount}</div>
      </td>
      <td className="px-4 lg:px-10 py-4">
        <TransactionTooltip tokens={transaction.tokens} side={isFirstTwoRows ? "bottom" : "top"}>
          <div className="flex -space-x-2 overflow-hidden cursor-pointer shrink-0">
            {transaction.tokens.slice(0, 5).map((token, index) => (
              <div
                key={index}
                className="relative w-6 h-6 lg:w-8 lg:h-8 rounded-full border-2 border-[#12131A] overflow-hidden bg-[#1A1D29] shrink-0"
              >
                <Image
                  src={token.icon}
                  alt={token.ticker}
                  fill
                  className="object-cover"
                />
              </div>
            ))}
            {transaction.tokens.length > 5 && (
              <div className="relative w-6 h-6 lg:w-8 lg:h-8 rounded-full border-2 border-[#12131A] bg-[#1A1D29] flex items-center justify-center text-[8px] lg:text-[10px] text-white font-bold shrink-0">
                +{transaction.tokens.length - 5}
              </div>
            )}
          </div>
        </TransactionTooltip>
      </td>
      <td className="px-4 lg:px-10 py-4">
        <div className="text-white font-medium text-sm lg:text-base">{transaction.members}</div>
      </td>
      <td className="px-4 lg:px-10 py-4">
        <div className="text-white text-sm lg:text-base">{transaction.time}</div>
      </td>
      <td className="px-4 lg:px-10 py-4">
        <div className="text-white text-sm lg:text-base">{transaction.date}</div>
      </td>
    </tr>

    {/* Mobile Card View (<768px) */}
    <tr className="md:hidden border-b border-white/5">
      <td colSpan={7} className="p-4">
        <div className="bg-[#1A1D29]/50 rounded-xl p-4 space-y-3">
          {/* Group Name & Amount */}
          <div className="flex items-center justify-between">
            <div className="text-white font-semibold text-base">{transaction.groupName}</div>
            <div className="text-white font-bold text-lg">{transaction.totalAmount}</div>
          </div>

          {/* Group Address */}
          <div className="flex items-center justify-between text-sm">
            <span className="text-[#A0A5BA]">Address:</span>
            <span className="text-white font-mono" title={transaction.groupAddress}>
              {truncateAddress(transaction.groupAddress, 12)}
            </span>
          </div>

          {/* Amount Breakdown */}
          <div className="flex items-center justify-between">
            <span className="text-[#A0A5BA] text-sm">Tokens:</span>
            <TransactionTooltip tokens={transaction.tokens} side="bottom">
              <div className="flex -space-x-2 overflow-hidden cursor-pointer shrink-0">
                {transaction.tokens.slice(0, 4).map((token, index) => (
                  <div
                    key={index}
                    className="relative w-7 h-7 rounded-full border-2 border-[#12131A] overflow-hidden bg-[#1A1D29] shrink-0"
                  >
                    <Image
                      src={token.icon}
                      alt={token.ticker}
                      fill
                      className="object-cover"
                    />
                  </div>
                ))}
                {transaction.tokens.length > 4 && (
                  <div className="relative w-7 h-7 rounded-full border-2 border-[#12131A] bg-[#1A1D29] flex items-center justify-center text-[9px] text-white font-bold shrink-0">
                    +{transaction.tokens.length - 4}
                  </div>
                )}
              </div>
            </TransactionTooltip>
          </div>

          {/* Members, Time, Date */}
          <div className="grid grid-cols-3 gap-3 pt-2 border-t border-white/5">
            <div className="text-center">
              <div className="text-[#A0A5BA] text-xs mb-1">Members</div>
              <div className="text-white font-medium text-sm">{transaction.members}</div>
            </div>
            <div className="text-center">
              <div className="text-[#A0A5BA] text-xs mb-1">Time</div>
              <div className="text-white text-xs">{transaction.time}</div>
            </div>
            <div className="text-center">
              <div className="text-[#A0A5BA] text-xs mb-1">Date</div>
              <div className="text-white text-xs">{transaction.date}</div>
            </div>
          </div>
        </div>
      </td>
    </tr>
  </>
);
}