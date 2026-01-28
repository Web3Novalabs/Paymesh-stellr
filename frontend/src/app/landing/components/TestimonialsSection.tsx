"use client";

import bricks from "../../../../public/bricks.svg";
import { motion } from "framer-motion";

const testimonials = [
  {
    name: "Paymesh User",
    handle: "@paymesh_defi",
    text: "Somewhere out there, someone&apos;s working hard for a dream—and this community is quietly helping them get there.",
  },
  {
    name: "Paymesh User",
    handle: "@paymesh_defi",
    text: "The AgentNation Discord isn't huge but is growing, little by little, through people who believe in possibilities. 🌍",
  },
  {
    name: "Paymesh User",
    handle: "@paymesh_defi",
    text: "Every contribution tells a story of support, of connection, of what&apos;s possible when we build together.",
  },
  {
    name: "Paymesh User",
    handle: "@paymesh_defi",
    text: "Somewhere out there, someone&apos;s working hard for a dream—and this community is quietly helping them get there.",
  },
  {
    name: "Paymesh User",
    handle: "@paymesh_defi",
    text: "The AgentNation Discord isn&apos;t huge but is growing, little by little, through people who believe in possibilities. 🌍",
  },
  {
    name: "Paymesh User",
    handle: "@paymesh_defi",
    text: "Every contribution tells a story of support, of connection, of what&apos;s possible when we build together.",
  },
];

export default function TestimonialsSection() {
  const containerVariants = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    show: { opacity: 1, y: 0, transition: { duration: 0.5 } },
  };

  return (
    <section className="w-full py-20 px-4 md:px-12 relative overflow-hidden">
      <div
        className="absolute inset-0 z-0 opacity-20 pointer-events-none bg-top bg-cover"
        style={{
          backgroundImage: `url(${bricks.src})`,
        }}
      ></div>

      <motion.div
        className="max-w-7xl mx-auto relative z-10"
        initial={{ opacity: 0, y: 30 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true }}
        transition={{ duration: 0.8 }}
      >
        <motion.h2
          className="text-2xl md:text-[44px] font-black text-center text-white uppercase mb-12 tracking-wide font-anton"
          initial={{ opacity: 0, y: -20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          BRICK BY BRICK WE GAIN THEIR TRUST
        </motion.h2>

        <motion.div
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
          variants={containerVariants}
          initial="hidden"
          whileInView="show"
          viewport={{ once: true, margin: "-50px" }}
        >
          {testimonials.map((t, i) => (
            <motion.div
              key={i}
              className="bg-[#0E0F19] border border-[#232542] p-6 rounded-xl hover:border-[#5B63D6] transition-colors"
              variants={itemVariants}
            >
              <div className="flex items-center gap-3 mb-4">
                <div className="w-10 h-10 bg-[#5B63D6] rounded-full flex items-center justify-center font-bold text-white">
                  P
                </div>
                <div>
                  <div className="text-white font-bold text-sm">{t.name}</div>
                  <div className="text-[#8398AD] text-xs">{t.handle}</div>
                </div>
              </div>
              <p className="text-[#E2E2E2] text-sm leading-relaxed">{t.text}</p>
            </motion.div>
          ))}
        </motion.div>
      </motion.div>
    </section>
  );
}
