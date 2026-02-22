"use client";

import React from "react";
import Link from "next/link";
import MovieIcon from "@/components/icons/movie";
import { motion } from "framer-motion";

export default function HeroSection() {
  return (
    <motion.section
      className="relative w-full flex flex-col items-center justify-center text-center pt-[170px] pb-20"
      initial={{ opacity: 0, y: 50 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.8 }}
    >
      <div className="max-w-3xl mx-auto space-y-1">
        <motion.h1
          className="text-3xl sm:text-4xl font-anton lg:text-[90px] font-black text-white tracking-tight uppercase"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.2, duration: 0.8 }}
        >
          REVOLUTIONIZING HOW <br />
          <span className="text-white">MONEY IS SHARED</span>
        </motion.h1>
        <motion.p
          className="text-[#9EB3C9] text-sm md:text-base font-extrabold tracking-wide uppercase"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4, duration: 0.8 }}
        >
          GET PAID AND WATCH THE MAGIC OF PAYMESH
        </motion.p>

        <motion.div
          className="flex flex-wrap items-center justify-center gap-4 pt-4"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.6, duration: 0.8 }}
        >
          <Link
            href="/user/overview"
            className="bg-[#5B63D6] hover:bg-[#4B53C6] text-white px-8 py-3 rounded-full text-sm font-bold transition-all shadow-[0_0_20px_rgba(91,99,214,0.3)] hover:shadow-[0_0_40px_rgba(91,99,214,0.5)]"
          >
            LAUNCH APP
          </Link>

          <button className="flex items-center gap-2 border border-[#2A2D45] hover:bg-[#FFFFFF05] text-[#E2E2E2] px-8 py-3 rounded-full text-sm font-bold transition-all">
            <span>DEMO</span>
            <MovieIcon />
          </button>
        </motion.div>
      </div>
    </motion.section>
  );
}
