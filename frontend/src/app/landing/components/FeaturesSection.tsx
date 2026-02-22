"use client";

import usdt from "../../../../public/coin/Image.png";
import usdc from "../../../../public/coin/Image (1).png";
import wbtc from "../../../../public/coin/Image (3).png";
import strk from "../../../../public/coin/Image (2).png";
import eth from "../../../../public/coin/Image (4).png";
import Image from "next/image";
import { motion } from "framer-motion";

export default function FeaturesSection() {
  const containerVariants = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: {
        staggerChildren: 0.2,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 30 },
    show: { opacity: 1, y: 0, transition: { duration: 0.6 } },
  };

  return (
    <motion.section
      className="w-full py-12 px-4 md:px-12 max-w-sit-screen mx-auto"
      variants={containerVariants}
      initial="hidden"
      whileInView="show"
      viewport={{ once: true, margin: "-100px" }}
    >
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <motion.div
          className="bg-[#0E0F19] border border-[#232542] rounded-lg p-8 flex flex-col h-full hover:border-[#5B63D6] transition-colors group"
          variants={itemVariants}
        >
          <div className="mb-6">
            <h3 className="text-[28px] font-anton font-bold text-[#DFDFE0] uppercase mb-2">
              GROUPS SPLIT
            </h3>
            <p className="text-[#FFFFFF] text-base font-dmsans">
              Create a group, add member wallet addresses, and let Paymesh do
              the rest. Every group gets a unique wallet address where funds can
              be sent and automatically distributed according to pre-set
              percentages. No spreadsheets, no manual tracking—just transparent,
              automated payments that flow exactly as intended.
            </p>
          </div>

          <div className="mt-auto space-y-4 font-anton text-base">
            <div className="p-px bg-linear-to-r to-[#00875A] from-[#005136] rounded-lg">
              <div className="rounded-lg  from-[#00B177] to-[#004B32] bg-linear-to-r p-6 flex items-center justify-between relative overflow-hidden">
                <span className="text-white text-sm uppercase relative z-10">
                  CREATE OR JOIN A GROUP
                </span>
                <Image
                  width={100}
                  className="absolute right-10 bottom-0 z-0"
                  height={100}
                  src={usdt}
                  alt=""
                />
              </div>
            </div>

            <div className="p-px bg-linear-to-r to-[#09479E] from-[#04224C] rounded-lg ">
              <div className="rounded-lg  from-[#0B4FB0] to-[#05214A] bg-linear-to-r p-6 flex items-center justify-end relative overflow-hidden">
                <span className="text-white text-sm uppercase relative z-10">
                  Get paid through Paymesh assigned group address
                </span>
                <Image
                  width={100}
                  className="absolute left-1 bottom-0 z-0"
                  height={100}
                  src={usdc}
                  alt=""
                />
              </div>
            </div>

            <div className="p-px bg-linear-to-r to-[#2E476C] from-[#5073B9] rounded-lg">
              <div className="rounded-lg  from-[#282664] to-[#282664] bg-linear-to-r p-6 flex items-center justify-between relative overflow-hidden">
                <span className="text-white text-sm uppercase relative z-10">
                  Token auto split by set percentage on payment
                </span>
                <Image
                  width={100}
                  className="absolute right-10 bottom-0 z-0"
                  height={100}
                  src={strk}
                  alt=""
                />
              </div>
            </div>
          </div>
        </motion.div>

        <motion.div
          className="bg-[#0E0F19] border border-[#232542] rounded-lg p-8 flex flex-col h-full hover:border-[#5B63D6] transition-colors group"
          variants={itemVariants}
        >
          <div className="mb-6">
            <h3 className="text-[28px] font-anton font-bold text-[#DFDFE0] uppercase mb-2">
              FUNDRAISING
            </h3>
            <p className="text-[#FFFFFF] text-base font-dmsans">
              Start a fundraiser, set your target, and let Paymesh handle the
              flow. Each campaign gets a unique wallet address to receive
              contributions on-chain. Once your goal is reached, funds are
              automatically sent to the designated beneficiary wallets—secure,
              transparent, and fully automated from start to finish.
            </p>
          </div>

          <div className="mt-auto space-y-4 font-anton text-base">
            <div className="p-px bg-linear-to-r to-[#09479E] from-[#04224C] rounded-lg">
              <div className="rounded-lg  from-[#FFFFFF] to-[#FFFFFF] bg-linear-to-r p-6 flex items-center justify-between relative overflow-hidden">
                <span className="text-[#111827] text-sm uppercase relative z-10">
                  start a fundraiser, set your target
                </span>
                <Image
                  width={100}
                  className="absolute right-10 bottom-0 z-0"
                  height={100}
                  src={usdc}
                  alt=""
                />
              </div>
            </div>

            <div className="p-px bg-linear-to-r to-[#838383] from-[#838383] rounded-lg">
              <div className="rounded-lg  from-[#660BB0] to-[#2B054A] bg-linear-to-r p-6 flex items-center justify-end relative overflow-hidden">
                <span className="text-white text-sm uppercase relative z-10">
                  get a unique wallet address to receive contributions
                </span>
                <Image
                  width={100}
                  className="absolute left-1 bottom-0 z-0"
                  height={100}
                  src={wbtc}
                  alt=""
                />
              </div>
            </div>

            <div className="p-px bg-linear-to-r to-[#09479E] from-[#04224C] rounded-lg">
              <div className="rounded-lg  from-[#E4C7B4] to-[#E4C7B4] bg-linear-to-r p-6 flex items-center justify-between relative overflow-hidden">
                <span className="text-[#161D63] text-sm uppercase relative z-10">
                  reach goal - receive funds to beneficiary wallets
                </span>
                <Image
                  width={100}
                  className="absolute right-10 bottom-0 z-0"
                  height={100}
                  src={eth}
                  alt=""
                />
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </motion.section>
  );
}
