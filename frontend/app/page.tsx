"use client";

import React from "react";
import HeroSection from "./landing/components/HeroSection";
import IntroSection from "./landing/components/IntroSection";
import FeaturesSection from "./landing/components/FeaturesSection";
import TestimonialsSection from "./landing/components/TestimonialsSection";
import StatsSection from "./landing/components/StatsSection";
import FaqSection from "./landing/components/FaqSection";
import Footer from "./landing/components/Footer";

export default function Page() {
  return (
    <main className="min-h-screen w-full text-white overflow-x-hidden">
      <HeroSection />
      <div className="py-25">
        <IntroSection />
        <FeaturesSection />
      </div>
      <TestimonialsSection />
      <StatsSection />
      <FaqSection />
      <Footer />
    </main>
  );
}
