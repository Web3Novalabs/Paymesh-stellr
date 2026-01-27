"use client";

import React, { useState } from "react";
import { Plus } from "lucide-react";
import typhoon from "../../../../public/coin/Image (5).png";
import stellar from "../../../../public/stellar.jpeg";
import Image from "next/image";
import { motion, AnimatePresence } from "framer-motion";

const faqData = [
  {
    question: "What is Paymesh?",
    answer:
      "Paymesh is an automated platform built with transparency in mind, allowing users to seamlessly create groups for split payments or manage fundraising campaigns on-chain.",
  },
  {
    question: "How does groups work in Paymesh?",
    answer:
      "You create a group, add member wallet addresses, and define split percentages. Paymesh generates a unique address for the group. Any funds sent to this address are automatically distributed to members according to the set rules.",
  },
  {
    question: "The idea behind Fundraiser?",
    answer:
      "Fundraisers on Paymesh get a unique wallet address. Donors send funds, and once the target is reached (or depending on configuration), funds are disbursed to the beneficiary wallets transparently.",
  },
  {
    question: "The users?",
    answer:
      "Paymesh is designed for anyone needing transparent, automated financial coordination—communities, DAOs, friends splitting costs, or charitable causes.",
  },
];

export default function FaqSection() {
  const [openIndex, setOpenIndex] = useState<number | null>(null);

  const toggle = (index: number) => {
    setOpenIndex(openIndex === index ? null : index);
  };

  return (
    <section className="w-full py-20 px-4 md:px-12 max-w-sit-screen mx-auto">
      <div className="bg-[#24225A] rounded-t-lg p-8 md:p-12 flex flex-col lg:flex-row gap-12 items-center">
        <motion.div
          className="lg:w-1/3"
          initial={{ opacity: 0, x: -30 }}
          whileInView={{ opacity: 1, x: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8 }}
        >
          <h2 className="text-3xl font-anton md:text-[36px] font-black text-white uppercase mb-4">
            FEQUENTLY ASKED QUESTIONS
          </h2>
          <p className="text-[#9EB3C9] text-sm uppercase font-extrabold">
            GET THE BASIC ANSWERS TO YOUR QUESTIONS HERE
          </p>
        </motion.div>

        <div className="lg:w-2/3 space-y-4">
          {faqData.map((item, index) => (
            <motion.div
              key={index}
              className="border-b border-[#FFFFFF1A] last:border-0 pb-4 last:pb-0 cursor-pointer"
              onClick={() => toggle(index)}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1, duration: 0.5 }}
            >
              <div className="flex items-center justify-between py-2">
                <h3 className="text-[#FFFFFF] font-semibold text-base md:text-base">
                  {item.question}
                </h3>
                <motion.div
                  animate={{ rotate: openIndex === index ? 90 : 0 }}
                  transition={{ duration: 0.3 }}
                >
                  <Plus size={16} className="text-[#FFFFFF]" />
                </motion.div>
              </div>
              <AnimatePresence>
                {openIndex === index && (
                  <motion.div
                    initial={{ height: 0, opacity: 0, marginTop: 0 }}
                    animate={{ height: "auto", opacity: 1, marginTop: 8 }}
                    exit={{ height: 0, opacity: 0, marginTop: 0 }}
                    transition={{ duration: 0.3 }}
                    className="overflow-hidden"
                  >
                    <p className="text-[#8398AD] text-sm leading-relaxed">
                      {item.answer}
                    </p>
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
          ))}
        </div>
      </div>
      <motion.div
        className="bg-[#0E0F19] rounded-b-lg p-15 mx-auto mb-16 flex flex-wrap items-center justify-center md:justify-between gap-8"
        initial={{ opacity: 0, y: 30 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true }}
        transition={{ delay: 0.3, duration: 0.8 }}
      >
        <div className="flex items-center gap-8  transition-all">
          <span className="text-white font-bold text-lg flex items-center gap-2">
            <Image
              width={40}
              height={40}
              src={stellar}
              alt="typhoon"
              className="rounded-full"
            />
            STELLAR
          </span>
          <span className="text-white font-bold text-lg flex items-center gap-2">
            <Image width={40} height={40} src={typhoon} alt="typhoon" /> TYPHOON
          </span>
        </div>

        <div className="text-right">
          <span className="text-[#DFDFE0] text-[44px] uppercase tracking-wider block font-anton">
            BUILT WITH THE BEST SUPPORT
          </span>
        </div>
      </motion.div>
    </section>
  );
}
