"use client";

import { useState } from "react";
import { usePathname } from "next/navigation";
import { Menu, X, Waves } from "lucide-react";

export default function Nav() {
  const [isOpen, setIsOpen] = useState(false);
  const pathname = usePathname();

  const navItems = [
    { label: "OVERVIEW", href: "/user/overview" },
    { label: "GROUPS", href: "/user/groups" },
    { label: "FUNDRAISER", href: "/user/fundraiser" },
    { label: "TRANSACTIONS", href: "/user/transactions" },
    { label: "PROFILE ANALYTICS", href: "/user/profile-analytics" },
  ];

  const walletAddress = "0x1a4l_1adc3d";

  return (
    <nav className="bg-[#0a0a0a] border-b border-gray-800 sticky top-0 z-50">
      {/* Desktop and Mobile Container */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo Section */}
          <div className="flex items-center gap-2 ">
            <Waves className="w-5 h-5 text-white" />
            <span className="text-white font-bold text-lg tracking-wide">
              PAYMESH
            </span>
          </div>

          {/* Desktop Navigation - Only on large screens */}
          <div className="hidden lg:flex items-center gap-8">
            {navItems.map((item) => {
              const isActive = pathname === item.href;
              return (
                <a
                  key={item.label}
                  href={item.href}
                  className={`text-sm font-medium transition-colors ${
                    isActive
                      ? "bg-blue-600 text-white px-4 py-2 rounded-full"
                      : "text-gray-300 hover:text-white"
                  }`}
                >
                  {item.label}
                </a>
              );
            })}
          </div>

          {/* Wallet Address - Desktop only */}
          <div className="hidden lg:flex items-center">
            <div className="border border-gray-600 rounded-full px-4 py-2 text-white text-sm font-mono cursor-pointer hover:border-gray-400 transition-colors">
              {walletAddress}
            </div>
          </div>

          {/* Tablet & Mobile: Wallet Address and Menu Button */}
          <div className="lg:hidden flex items-center gap-2 sm:gap-3">
            <div className="border border-gray-600 rounded-full px-2 py-1 sm:px-3 sm:py-1.5 text-white text-xs font-mono truncate max-w-[100px] sm:max-w-none">
              {walletAddress}
            </div>
            <button
              onClick={() => setIsOpen(!isOpen)}
              className="text-white p-2 rounded-md hover:bg-gray-800 transition-colors flex-shrink-0"
            >
              {isOpen ? (
                <X className="w-6 h-6" />
              ) : (
                <Menu className="w-6 h-6" />
              )}
            </button>
          </div>
        </div>

        {/* Mobile Navigation Menu */}
        {isOpen && (
          <div className="lg:hidden pb-4 pt-2">
            <div className="flex flex-col gap-2">
              {navItems.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <a
                    key={item.label}
                    href={item.href}
                    className={`px-4 py-3 rounded-lg text-sm font-medium transition-colors ${
                      isActive
                        ? "bg-blue-600 text-white"
                        : "text-gray-300 hover:bg-gray-800 hover:text-white"
                    }`}
                    onClick={() => setIsOpen(false)}
                  >
                    {item.label}
                  </a>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </nav>
  );
}
