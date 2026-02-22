"use client";

import React from "react";
import { motion } from "framer-motion";
import { Navbar } from "@/components/Navbar";
import Image from "next/image";

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

function ViewAllButton() {
  return <button className="overview-view-all">View all</button>;
}

function StatusBadge({ status }: { status: GroupRow["status"] }) {
  const statusClass =
    status === "Active" ? "overview-status-active" : "overview-status-paid";

  return <span className={`overview-status-badge ${statusClass}`}>{status}</span>;
}

function ProgressPill({ progress }: { progress: number }) {
  return (
    <div className="overview-progress-pill">
      <div className="overview-progress-track">
        <div
          className="overview-progress-fill"
          style={{ width: `${progress}%` }}
        />
      </div>
      <span className="overview-progress-label">{progress}% Completed</span>
    </div>
  );
}

function InfoCard({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className="overview-card"
    >
      <div className="overview-card-head">
        <h2 className="overview-card-title">{title}</h2>
        <ViewAllButton />
      </div>
      {children}
    </motion.section>
  );
}

export default function Overview() {
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

      <main className="overview-layout">
        <section className="overview-content-wrap">
          <motion.div
            initial={{ opacity: 0, y: -12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.45 }}
            className="overview-coins-row"
          >
            {TOKEN_ICONS.map((icon, index) => (
              <Image
                key={icon}
                src={icon}
                alt="token"
                width={112}
                height={112}
                className={`overview-coin ${index === 1 ? "overview-coin-offset" : ""}`}
              />
            ))}
          </motion.div>

          <div className="overview-divider" />

          <div className="overview-cards-grid">
            <InfoCard title="Your Groups">
              <div className="overview-table-head overview-groups-grid">
                <span>Name</span>
                <span className="overview-align-center">Members</span>
                <span className="overview-align-right">Status</span>
              </div>

              <div className="overview-rows">
                {GROUP_ROWS.map((group, index) => (
                  <div key={`${group.name}-${index}`} className="overview-groups-grid">
                    <span className="overview-row-text">{group.name}</span>
                    <span className="overview-row-text overview-align-center">
                      {group.members}
                    </span>
                    <div className="overview-align-right">
                      <StatusBadge status={group.status} />
                    </div>
                  </div>
                ))}
              </div>
            </InfoCard>

            <InfoCard title="Active Crowd Funding">
              <div className="overview-table-head overview-funds-grid">
                <span>Name</span>
                <span className="overview-align-right">Progress</span>
              </div>

              <div className="overview-rows">
                {CROWD_FUNDING_ROWS.map((fund, index) => (
                  <div key={`${fund.name}-${index}`} className="overview-funds-grid overview-fund-row">
                    <span className="overview-row-text">{fund.name}</span>
                    <div className="overview-align-right">
                      <ProgressPill progress={fund.progress} />
                    </div>
                  </div>
                ))}
              </div>
            </InfoCard>
          </div>
        </section>
      </main>
    </>
  );
}
