"use client";
import { useState } from "react";
import EmptyState from "@/app/user/transactions/components/EmptyState";
import { TransactionData } from "@/app/user/transactions/components/TransactionRow";
import Pagination from "@/app/user/transactions/components/Pagination";
import { TransactionList } from "@/app/user/transactions/components/TransactionList";

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
  const totalPages = Math.ceil(TOTAL_ITEMS / ITEMS_PER_PAGE);
  const transactions = showEmpty ? [] : MOCK_TRANSACTIONS;

return (
  <>
    <div
      className="min-h-screen w-full fixed inset-0 -z-10"
      style={{
        backgroundImage: 'url("/Bg 1.svg")',
        backgroundSize: 'cover',
        backgroundPosition: 'center',
        backgroundRepeat: 'no-repeat',
        backgroundAttachment: 'fixed'
      }}
    />
    <div className="min-h-screen pt-16 sm:pt-20 lg:pt-24 pb-8 sm:pb-12 px-3 sm:px-4 lg:px-8 max-w-425 mx-auto relative z-10 w-full overflow-x-hidden">
      <div className="flex flex-col gap-4 sm:gap-6 lg:gap-8 w-full max-w-full">
        {transactions.length > 0 ? (
          <>
            <div className="bg-[#0A0B0F]/40 backdrop-blur-xl rounded-xl sm:rounded-2xl lg:rounded-3xl border border-white/10 shadow-2xl flex flex-col overflow-hidden p-2 sm:p-3 lg:p-4 min-h-[400px] sm:min-h-[500px] max-h-[400px] w-full">
              <div className="flex-1 w-full overflow-y-auto transaction-list-scrollable">
                <TransactionList transactions={transactions} />
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
    </div>
    </>
  );
}