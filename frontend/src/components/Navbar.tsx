"use client";

import { useState } from "react";
import { Menu, X } from "lucide-react";
import ConnectWalletButton from "./ConnectWalletButton";

const NAV_ITEMS = [
  { name: "Overview", href: "#" },
  { name: "Groups", href: "#" },
  { name: "Fundraiser", href: "#" },
  { name: "Transactions", href: "#" },
  { name: "Profile Analytics", href: "#" },
];

export function Navbar() {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  return (
    <nav className="fixed top-0 left-0 right-0 z-40 backdrop-blur-lg border-b border-white/5 flex items-center justify-between py-[14px] lg:py-5 px-3 lg:px-[100px]">
      {/* Logo */}
      <div className="flex items-center gap-[7px] py-1 pr-3 px-2 rounded-full border-[#232542] border-[1px]">
        <div className="relative w-9 h-9 lg:w-12 lg:h-12 overflow-hidden rounded-full border border-white/10">
          <img
            src="/logo.jpeg"
            alt="Paymesh Logo"
            className="w-full h-full object-cover"
          />
        </div>
        <span className="text-2xl/[100%] lg:text-[28px]/12 font-bold text-white tracking-wide font-anton">
          PAYMESH
        </span>
      </div>

      {/* Desktop Menu - Centered Pill */}
      <div className="hidden md:block absolute left-1/2 transform -translate-x-1/2">
        <div className="flex items-center p-1.5 bg-[#0D0D10] border border-[#232542] rounded-full">
          {NAV_ITEMS.map((item) => (
            <a
              key={item.name}
              href={item.href}
              className={`px-6 py-[15px] rounded-full text-[#dddddd] text-xs font-black tracking-[0] uppercase transition-colors ${
                item.name === "Overview" ? "bg-[#5B63D6]" : "hover:bg-[#5B63D6]"
              }`}
            >
              {item.name}
            </a>
          ))}
        </div>
      </div>

      {/* Connect Button (Desktop) */}
      {/* <div className="hidden md:block"> */}
      <ConnectWalletButton />
      {/* </div> */}

      {/* Mobile menu button */}
      <div className="flex md:hidden">
        <button
          onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
          className="inline-flex items-center justify-center p-3 rounded-full text-[#E2E2E2] bg-[#101011] focus:outline-none"
        >
          {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
        </button>
      </div>
    </nav>
  );
}
