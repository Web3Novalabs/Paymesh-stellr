"use client";

import React from "react";
import { Navbar } from "@/components/Navbar";
import ActiveGroupsWidget from "./components/ActiveGroupsWidget";
import { motion } from "framer-motion";
import { LayoutDashboard } from "lucide-react";

export default function OverviewPage() {
  return (
    <>
      <div
        className="min-h-screen w-full fixed inset-0 -z-10"
        style={{
          backgroundImage: 'url("/Bg 1.svg")',
          backgroundSize: "cover",
          backgroundPosition: "center",
          backgroundRepeat: "no-repeat",
          backgroundAttachment: "fixed",
        }}
      />
      <Navbar />
      <div className="min-h-screen pt-24 sm:pt-28 lg:pt-32 pb-8 sm:pb-12 px-3 sm:px-4 lg:px-8 max-w-5xl mx-auto relative z-10 w-full overflow-x-hidden">
        {/* Page Header */}
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="flex items-center gap-3 mb-6 sm:mb-8"
        >
          <div className="w-10 h-10 sm:w-12 sm:h-12 rounded-xl sm:rounded-2xl bg-gradient-to-br from-[#5B63D6]/20 to-[#5B63D6]/5 border border-[#5B63D6]/15 flex items-center justify-center">
            <LayoutDashboard className="w-5 h-5 sm:w-6 sm:h-6 text-[#8B92E8]" />
          </div>
          <div>
            <h1 className="text-white text-xl sm:text-2xl lg:text-3xl font-bold tracking-tight">
              Overview
            </h1>
            <p className="text-[#5A6578] text-xs sm:text-sm font-medium mt-0.5">
              Welcome back — here&apos;s a snapshot of your groups
            </p>
          </div>
        </motion.div>

        {/* Widgets Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-1 gap-6 sm:gap-8">
          <ActiveGroupsWidget />
        </div>
      </div>
    </>
  );
}
