"use client";

import React from "react";
import { motion } from "framer-motion";

type GroupRow = {
  name: string;
  members: number;
  status: "Active" | "Paid";
};

type CrowdFundingRow = {
  name: string;
  progress: number;
};

const GROUP_ROWS: GroupRow[] = [
  { name: "Thebuidathon", members: 6, status: "Active" },
  { name: "Thebuidathon", members: 6, status: "Active" },
  { name: "Hack4you", members: 3, status: "Paid" },
  { name: "DevSquad", members: 2, status: "Paid" },
];

const CROWD_FUNDING_ROWS: CrowdFundingRow[] = [
  { name: "ARG Trip", progress: 70 },
  { name: "ARG Trip", progress: 70 },
  { name: "Project", progress: 50 },
  { name: "School", progress: 20 },
];

const TOKEN_ICONS = [
  "/coin/Image (3).png",
  "/coin/Image (4).png",
  "/strkImg.png",
  "/coin/Image (5).png",
  "/usdtImg.png",
];

function StatusBadge({ status }: { status: GroupRow["status"] }) {
  const isActive = status === "Active";

  return (
    <span
      className={`inline-flex h-7 min-w-[58px] items-center justify-center rounded-full border px-3 text-[11px] font-medium ${
        isActive
          ? "border-emerald-400/15 bg-emerald-500/10 text-emerald-400"
          : "border-indigo-300/15 bg-indigo-500/10 text-indigo-300"
      }`}
    >
      {status}
    </span>
  );
}

function ProgressPill({ progress }: { progress: number }) {
  return (
    <div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-[#10152d]/70 px-2.5 py-1">
      <div className="h-1.5 w-9 overflow-hidden rounded-full bg-white/30">
        <div
          className="h-full rounded-full bg-[#5d63ea]"
          style={{ width: `${progress}%` }}
        />
      </div>
      <span className="text-[11px] text-white/60">{progress}% Completed</span>
    </div>
  );
}

function ViewAllButton() {
  return (
    <button className="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-[#E2E7FF] transition hover:bg-white/10">
      View all
    </button>
  );
}

export default function OverviewPage() {
  return (
    <div className="mx-auto w-full max-w-[980px] pt-2 sm:pt-4">
      <motion.div
        initial={{ opacity: 0, y: -12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.45 }}
        className="mb-4 flex justify-center gap-3 overflow-x-auto pb-4 scrollbar-hide sm:mb-6 sm:gap-4"
      >
        {TOKEN_ICONS.map((icon, index) => (
          <img
            key={icon}
            src={icon}
            alt="token"
            className={`h-20 w-20 rounded-full border border-white/20 object-cover shadow-[0_8px_24px_rgba(0,0,0,0.4)] sm:h-24 sm:w-24 lg:h-28 lg:w-28 ${
              index === 1 ? "translate-y-[2px]" : ""
            }`}
          />
        ))}
      </motion.div>

      <div className="mb-5 h-px w-full bg-white/50 sm:mb-7" />

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <motion.section
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
          className="rounded-xl border border-white/10 bg-[linear-gradient(120deg,rgba(20,27,54,0.78),rgba(8,12,28,0.78))] p-4 backdrop-blur-xl sm:p-5"
        >
          <div className="mb-5 flex items-center justify-between">
            <h2 className="text-2xl font-medium leading-tight text-white sm:text-[30px]">
              Your Groups
            </h2>
            <ViewAllButton />
          </div>

          <div className="mb-3 grid grid-cols-[1.5fr_0.7fr_0.8fr] items-center text-xs text-[#94a4c1]">
            <span>Name</span>
            <span className="text-center">Members</span>
            <span className="text-right">Status</span>
          </div>

          <div className="space-y-3">
            {GROUP_ROWS.map((group, index) => (
              <div
                key={`${group.name}-${index}`}
                className="grid grid-cols-[1.5fr_0.7fr_0.8fr] items-center"
              >
                <span className="text-sm text-white/90">{group.name}</span>
                <span className="text-center text-sm text-white/90">
                  {group.members}
                </span>
                <div className="text-right">
                  <StatusBadge status={group.status} />
                </div>
              </div>
            ))}
          </div>
        </motion.section>

        <motion.section
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: 0.05 }}
          className="rounded-xl border border-white/10 bg-[linear-gradient(120deg,rgba(20,27,54,0.78),rgba(8,12,28,0.78))] p-4 backdrop-blur-xl sm:p-5"
        >
          <div className="mb-5 flex items-center justify-between">
            <h2 className="text-2xl font-medium leading-tight text-white sm:text-[30px]">
              Active Crowd Funding
            </h2>
            <ViewAllButton />
          </div>

          <div className="mb-3 grid grid-cols-[1fr_1.4fr] items-center gap-3 text-xs text-[#94a4c1]">
            <span>Name</span>
            <span className="text-right">Progress</span>
          </div>

          <div className="space-y-3">
            {CROWD_FUNDING_ROWS.map((fund, index) => (
              <div
                key={`${fund.name}-${index}`}
                className="grid min-h-8 grid-cols-[1fr_1.4fr] items-center gap-3"
              >
                <span className="text-sm text-white/90">{fund.name}</span>
                <div className="text-right">
                  <ProgressPill progress={fund.progress} />
                </div>
              </div>
            ))}
          </div>
        </motion.section>
      </div>

      <style jsx>{`
        .scrollbar-hide {
          -ms-overflow-style: none;
          scrollbar-width: none;
        }
        .scrollbar-hide::-webkit-scrollbar {
          display: none;
        }
      `}</style>
    </div>
  );
}
