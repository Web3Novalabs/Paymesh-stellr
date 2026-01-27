import type { Metadata } from "next";
import { DM_Sans, Geist, Geist_Mono, Anton } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

const anton = Anton({
  weight: ["400"], // Anton only has one weight
  subsets: ["latin"],
  display: "swap",
  variable: "--font-anton",
});

const dmSans = DM_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "700", "900"], // Choose the weights you need
  display: "swap",
  variable: "--font-dmsans",
});

export const metadata: Metadata = {
  title: "PAYMESH",
  description: "An automated group payment on stellar",
  keywords: [
    "decentralized security",
    "paymesh",
    "payment",
    "security",
    "automated rewards",
    "trustless",
    "Web3 payment",
    "crypto payment",
  ],
  openGraph: {
    title: "PAYMESH - An automated group payment on stellar",
    description:
      "Paymesh automates group payment distribution using Stellar smart contracts. Create a group, set wallet addresses with specific percentages, and any payment sent to your group address automatically splits and distributes funds instantly,",
    url: "https://paymesh.app",
    siteName: "paymesh",
    images: [
      {
        url: "https://paymesh.app/logo.jpeg",
        width: 1200,
        height: 630,
        alt: "paymesh - Decentralized Payment Platform",
      },
    ],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "PAYMESH - An automated group payment on stellar",
    description:
      "Paymesh automates group payment distribution using Starknet smart contracts. Create a group, set wallet addresses with specific percentages, and any payment sent to your group address automatically splits and distributes funds instantly,",
    images: ["https://paymesh.app/logo.jpeg"],
    creator: "@paymeshglobal",
  },

  icons: {
    icon: [
      { url: "/Group 1.svg" },
      {
        url: "/Group 1.svg",
        sizes: "192x192",
        type: "image/svg+xml",
      },
      {
        url: "/Group 1.svg",
        sizes: "512x512",
        type: "image/svg+xml",
      },
    ],
    apple: [
      {
        url: "/Group 1.svg",
        sizes: "180x180",
        type: "image/svg+xml",
      },
    ],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`bg-no-repeat bg-fixed bg h-full bg-cover ${dmSans.variable} ${anton.variable} ${geistSans.variable} ${geistMono.variable} antialiased font-dmsans`}
        suppressHydrationWarning={true}
      >
        {children}
      </body>
    </html>
  );
}
