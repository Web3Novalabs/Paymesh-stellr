"use client";

import React, { useState } from "react";
import HeroSection from "./landing/components/HeroSection";
import IntroSection from "./landing/components/IntroSection";
import FeaturesSection from "./landing/components/FeaturesSection";
import TestimonialsSection from "./landing/components/TestimonialsSection";
import StatsSection from "./landing/components/StatsSection";
import FaqSection from "./landing/components/FaqSection";
import Footer from "./landing/components/Footer";
import { Navbar } from "@/components/Navbar";
import { SharePopup } from "@/components/share-popup";
import { Share2 } from "lucide-react";

export default function Page() {
  const [isSharePopupOpen, setIsSharePopupOpen] = useState(false);

  return (
    <main className="min-h-screen w-full text-white overflow-x-hidden">
      <Navbar />
      <main>
        <HeroSection />
        <div className="py-25">
          <IntroSection />
          <FeaturesSection />
        </div>
        <TestimonialsSection />
        <StatsSection />
        <FaqSection />
      </main>
      <Footer />

      {/* Test Button for Share Popup - Preview Feature */}
      <button
        type="button"
        onClick={() => setIsSharePopupOpen(true)}
        className="fixed bottom-6 right-6 z-40 bg-linear-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 text-white p-4 rounded-full shadow-2xl transition-all transform hover:scale-110 flex items-center gap-2 group"
        aria-label="Test share popup"
        title="Preview Share Popup"
      >
        <Share2 size={24} />
        <span className="hidden group-hover:inline-block text-sm font-semibold pr-2">
          Test Share
        </span>
      </button>

      <SharePopup
        isOpen={isSharePopupOpen}
        onClose={() => setIsSharePopupOpen(false)}
        amountRaised={2500}
        targetAmount={5000}
      />
    </main>
  );
}
