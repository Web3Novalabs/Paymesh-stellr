"use client";
import { useState, useEffect } from "react";
import EmptyState from "./components/EmptyState";
import { TransactionData } from "./components/TransactionRow";
import Pagination from "./components/Pagination";
import { TransactionList } from "./components/TransactionList";
import { Skeleton } from "@/src/components/ui/skeleton";

function TransactionRowSkeleton() {
  return (
    <div className="flex items-center gap-4 p-3 border-b border-white/5">
      <div className="flex-1 space-y-2">
        <Skeleton className="h-4 w-24 bg-white/10" />
        <Skeleton className="h-3 w-32 bg-white/10" />
      </div>
      <Skeleton className="h-4 w-12 bg-white/10" />
    </div>
  );
}

const MOCK_TRANSACTIONS: TransactionData[] = [
  {
    id: "1",
    groupName: "Paymesh",
    groupAddress: "0x6B8e...7cF3",
    totalAmount: "$120",
    members: 4,
    time: "4:45:08PM",
    date: "Sep 9th, 2025",
    tokens: [
      { name: "Bitcoin", ticker: "wBTC", amount: "0.00016", icon: "/coin/Image (3).png", value: "$60" },
      { name: "Ethereum", ticker: "ETH", amount: "0.01", icon: "/coin/Image (4).png", value: "$30" },
      { name: "Stellar", ticker: "XLM", amount: "500", icon: "/Stellar.png", value: "$10" },
      { name: "USDC", ticker: "USDC", amount: "65", icon: "/coin/Image (1).png", value: "$10" },
      { name: "USDT", ticker: "USDT", amount: "35", icon: "/coin/Image.png", value: "$10" },
    ],
  },
  {
    id: "2",
    groupName: "TechFlow",
    groupAddress: "0x8D7f...2aB1",
    totalAmount: "$250",
    members: 2,
    time: "3:15:32PM",
    date: "Oct 2nd, 2025",
    tokens: [
      { name: "Bitcoin", ticker: "wBTC", amount: "0.00016", icon: "/coin/Image (3).png" },
      { name: "Ethereum", ticker: "ETH", amount: "0.01", icon: "/coin/Image (4).png" },
      { name: "Stellar", ticker: "XLM", amount: "500", icon: "/Stellar.png" },
      { name: "USDC", ticker: "USDC", amount: "$65", icon: "/coin/Image (1).png" },
      { name: "USDT", ticker: "USDT", amount: "$35", icon: "/coin/Image.png" },
    ],
  },
  {
    id: "3",
    groupName: "Greenwave",
    groupAddress: "0x4F2b...7eD9",
    totalAmount: "$90",
    members: 5,
    time: "1:30:45PM",
    date: "Oct 10, 2025",
    tokens: [
      { name: "Bitcoin", ticker: "wBTC", amount: "0.00016", icon: "/coin/Image (3).png" },
      { name: "Ethereum", ticker: "ETH", amount: "0.01", icon: "/coin/Image (4).png" },
      { name: "USDC", ticker: "USDC", amount: "$65", icon: "/coin/Image (1).png" },
      { name: "USDT", ticker: "USDT", amount: "$35", icon: "/coin/Image.png" },
    ],
  },
  {
    id: "4",
    groupName: "CrypotoNest",
    groupAddress: "0x9C1d...6FE4",
    totalAmount: "$300",
    members: 3,
    time: "12:00:00PM",
    date: "Nov 15, 2025",
    tokens: [
      { name: "Bitcoin", ticker: "wBTC", amount: "0.00016", icon: "/coin/Image (3).png" },
      { name: "Ethereum", ticker: "ETH", amount: "0.01", icon: "/coin/Image (4).png" },
      { name: "Stellar", ticker: "XLM", amount: "500", icon: "/Stellar.png" },
      { name: "USDC", ticker: "USDC", amount: "$65", icon: "/coin/Image (1).png" },
      { name: "USDT", ticker: "USDT", amount: "$35", icon: "/coin/Image.png" },
    ],
  },
  {
    id: "5",
    groupName: "DataPulse",
    groupAddress: "Ox2B3c...5FC8",
    totalAmount: "$75",
    members: 6,
    time: "5:3012PM",
    date: "Dec Ist, 2025",
    tokens: [
      { name: "Stellar", ticker: "XLM", amount: "500", icon: "/Stellar.png" },
      { name: "USDC", ticker: "USDC", amount: "$65", icon: "/coin/Image (1).png" },
      { name: "USDT", ticker: "USDT", amount: "$35", icon: "/coin/Image.png" },
    ],
  },
  {
    id: "6",
    groupName: "Revest",
    groupAddress: "0x2B3c...5FC8",
    totalAmount: "$75",
    members: 6,
    time: "5:30:12PM",
    date: "Dec Ist, 2025",
    tokens: [
      { name: "Bitcoin", ticker: "wBTC", amount: "0.00016", icon: "/coin/Image (3).png" },
      { name: "Ethereum", ticker: "ETH", amount: "0.01", icon: "/coin/Image (4).png" },
      { name: "Stellar", ticker: "XLM", amount: "500", icon: "/Stellar.png" },
      { name: "USDC", ticker: "USDC", amount: "$65", icon: "/coin/Image (1).png" },
      { name: "USDT", ticker: "USDT", amount: "$35", icon: "/coin/Image.png" },
    ],
  },
];

const TOTAL_ITEMS = 50;
const ITEMS_PER_PAGE = 10;

export default function TransactionsPage() {
  const [currentPage, setCurrentPage] = useState(1);
  const [showEmpty] = useState(false); // Set to false to show transactions by default
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => setIsLoading(false), 2000);
    return () => clearTimeout(timer);
  }, []);

  const totalPages = Math.ceil(TOTAL_ITEMS / ITEMS_PER_PAGE);
  const transactions = showEmpty ? [] : MOCK_TRANSACTIONS;

  return (
    <div className="flex flex-col gap-4 sm:gap-6 lg:gap-8 w-full max-w-full">
      {transactions.length > 0 ? (
        <>
          <div className="bg-[#0A0B0F]/40 backdrop-blur-xl rounded-xl sm:rounded-2xl lg:rounded-3xl border border-white/10 shadow-2xl flex flex-col overflow-hidden p-2 sm:p-3 lg:p-4 min-h-[400px] sm:min-h-[500px] max-h-[400px] w-full">
            <div className="flex-1 w-full overflow-y-auto transaction-list-scrollable">
              {isLoading ? (
                <div className="divide-y divide-white/5">
                  {[1, 2, 3, 4, 5, 6].map((i) => (
                    <TransactionRowSkeleton key={i} />
                  ))}
                </div>
              ) : (
                <TransactionList transactions={transactions} />
              )}
            </div>
          </div>
          <div className="w-full max-w-full">
            <Pagination
              currentPage={currentPage}
              totalPages={totalPages}
              totalItems={TOTAL_ITEMS}
              itemsPerPage={ITEMS_PER_PAGE}
              onPageChange={setCurrentPage}
            />
          </div>
        </>
      ) : (
        <div className="w-full">
          <EmptyState />
        </div>
      )}
    </div>
  );
}