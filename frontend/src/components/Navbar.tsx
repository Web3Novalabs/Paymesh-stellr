"use client";

import { useState } from "react";
import { Menu, X } from "lucide-react";
import ConnectWalletButton from "./ConnectWalletButton";

export function Navbar() {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  return (
    <nav className="fixed top-0 left-0 right-0 z-40 bg-black/50 backdrop-blur-lg border-b border-white/10">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <div className="flex-shrink-0">
            <span className="text-2xl font-bold bg-gradient-to-r from-[#5B6FE8] to-purple-500 bg-clip-text text-transparent">
              PAYMESH
            </span>
          </div>

          {/* Desktop Menu */}
          <div className="hidden md:block">
            <div className="ml-10 flex items-baseline space-x-4">
              <a
                href="#"
                className="text-gray-300 hover:text-white px-3 py-2 rounded-md text-sm font-medium"
              >
                Dashboard
              </a>
              <a
                href="#"
                className="text-gray-300 hover:text-white px-3 py-2 rounded-md text-sm font-medium"
              >
                Transactions
              </a>
              <a
                href="#"
                className="text-gray-300 hover:text-white px-3 py-2 rounded-md text-sm font-medium"
              >
                Settings
              </a>
            </div>
          </div>

          {/* Connect Button (Desktop) */}
          <div className="hidden md:block cursor-pointer">
            <ConnectWalletButton />
          </div>

          {/* Mobile menu button */}
          <div className="-mr-2 flex md:hidden">
            <button
              onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
              className="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-700 focus:outline-none"
            >
              {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Menu */}
      {isMobileMenuOpen && (
        <div className="md:hidden bg-[#0B0E14] border-b border-white/10">
          <div className="px-2 pt-2 pb-3 space-y-1 sm:px-3">
            <a
              href="#"
              className="text-gray-300 hover:text-white block px-3 py-2 rounded-md text-base font-medium"
            >
              Dashboard
            </a>
            <a
              href="#"
              className="text-gray-300 hover:text-white block px-3 py-2 rounded-md text-base font-medium"
            >
              Transactions
            </a>
            <a
              href="#"
              className="text-gray-300 hover:text-white block px-3 py-2 rounded-md text-base font-medium"
            >
              Settings
            </a>
            <div className="mt-4 px-3">
              <ConnectWalletButton />
            </div>
          </div>
        </div>
      )}
    </nav>
  );
}
