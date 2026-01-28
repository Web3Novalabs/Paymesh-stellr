"use client";

export default function EmptyState() {

return (
  <div className="flex flex-col items-center justify-center py-12 sm:py-16 lg:py-20 px-4 sm:px-6 bg-[#12131A] rounded-2xl border border-white/10">
    <div className="w-16 h-16 sm:w-20 sm:h-20 mb-4 sm:mb-6 rounded-full bg-white/5 flex items-center justify-center">
      <svg
        className="w-8 h-8 sm:w-10 sm:h-10 text-white/30"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
    </div>
    <h3 className="text-xl sm:text-2xl font-bold text-white mb-2 sm:mb-3 text-center">
      No transactions yet
    </h3>
    <p className="text-[#A0A5BA] text-center max-w-md text-sm sm:text-base px-4">
      When you start making transactions, they will appear here with all the details including
      amounts, recipients, and timestamps.
    </p>
  </div>
);
}
