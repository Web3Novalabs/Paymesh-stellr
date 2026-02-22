"use client";

import React from "react";
import Link from "next/link";
import coins from "../../../../public/coin/Container (1).png";
import Image from "next/image";
import logo from "../../../../public/navLogo.svg";
import Tg from "@/components/icons/tg";
import X from "@/components/icons/x";

const gradientStops = "#ff0000, #00ff00";

export default function Footer() {
  return (
    <footer className="w-full pt-10 pb-4 px-4 md:px-12">
      <div className="mb-16 overflow-hidden">
        <Image className=" w-4/5 md:w-1/2 mx-auto" src={coins} alt="coins" />
        <div
          className="w-full h-px text-white"
          style={{
            background: "#000000",
            backgroundImage: `linear-gradient(#000000, #000000), linear-gradient(135deg, ${gradientStops})`,
            backgroundOrigin: "border-box",
            backgroundClip: "padding-box, border-box",
            border: "2px solid transparent",
          }}
        ></div>
      </div>

      <div className="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-6">
        {/* LOGO GROUP */}
        <Link
          href="/"
          className="flex items-center gap-3 border border-[#232542] rounded-full py-1 px-3 cursor-pointer z-50 relative"
        >
          <Image className="" src={logo} alt="paymesh logo" />
          <h1 className="text-base uppercase md:text-[28px] font-anton">
            Paymesh
          </h1>
        </Link>

        <div className="flex flex-col gap-y-3 w-full md:w-auto sm:flex-row md:gap-4 items-center">
          {/* Documentation Link */}
          <Link
            href="#"
            className="bg-[#FFFFFF0D] px-4 py-3 rounded-full text-xs text-[#8398AD] font-bold hover:text-white transition-colors gap-2 flex items-center justify-center w-full md:w-auto"
          >
            DOCUMENTATION <Tg />
          </Link>

          {/* X (Twitter) Link */}
          <Link
            href="https://x.com/paymesh_"
            target="_blank"
            className="bg-[#FFFFFF0D] px-4 py-3 rounded-full text-xs text-[#8398AD] font-bold hover:text-white transition-colors gap-2 flex items-center justify-center w-full md:w-auto"
          >
            X (TWITTER) <X />
          </Link>

          {/* Telegram Link */}
          <Link
            href="https://t.me/web3noval"
            target="_blank"
            className="bg-[#FFFFFF0D] px-4 py-3 rounded-full text-xs text-[#8398AD] font-bold hover:text-white transition-colors gap-2 flex items-center justify-center w-full md:w-auto"
          >
            TELEGRAM <Tg />
          </Link>
        </div>
      </div>
    </footer>
  );
}
