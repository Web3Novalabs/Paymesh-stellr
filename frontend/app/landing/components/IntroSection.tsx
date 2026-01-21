"use client";
import { motion } from "framer-motion";

export default function IntroSection() {
  return (
    <motion.section
      className="w-full px-4 text-center mx-auto"
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.8 }}
    >
      <h2 className="text-2xl font-anton font-normal md:text-3xl lg:text-4xl text-white uppercase mb-4 tracking-wide leading-tight">
        PAYMESH IS AN AUTOMATED PLATFORM BUILT WITH TRANSPARENCY IN MIND
      </h2>
      <p className="text-[#9EB3C9] md:text-base font-medium tracking-wider uppercase mx-auto">
        SEAMLESSLY CREATE AND JOIN GROUPS FOR SPLIT PAYMENT BETWEEN MEMBERS OR
        CREATE/DONATE TO FUNDRAISERS
      </p>
    </motion.section>
  );
}
