"use client";

import { ChevronLeft, ChevronRight } from "lucide-react";

interface PaginationProps {
  currentPage: number;
  totalPages: number;
  totalItems: number;
  itemsPerPage: number;
  onPageChange: (page: number) => void;
}

export default function Pagination({
  currentPage,
  totalPages,
  totalItems,
  itemsPerPage,
  onPageChange,
}: PaginationProps) {
  const getPageNumbers = () => {
    const pages: number[] = [];
    // Show pages in blocks of 3 (e.g., 1-3, then 4-6)
    // If currentPage is 1, 2, or 3 -> show 1, 2, 3
    // If currentPage is 4 or 5 -> show 4, 5
    const blockSize = 3;
    const currentBlock = Math.ceil(currentPage / blockSize);
    const startPage = (currentBlock - 1) * blockSize + 1;
    const endPage = Math.min(totalPages, startPage + blockSize - 1);

    for (let i = startPage; i <= endPage; i++) {
        pages.push(i);
    }
    return pages;
  };

  const pages = getPageNumbers();
  const startItem = (currentPage - 1) * itemsPerPage + 1;
  const endItem = Math.min(currentPage * itemsPerPage, totalItems);

return (
  <div className="flex flex-col sm:flex-row items-center justify-between gap-4 sm:gap-6 w-full px-2">
    {/* Showing text */}
    <div className="text-white text-xs sm:text-sm lg:text-base font-medium opacity-80 text-center sm:text-left">
      Showing <span className="text-white font-bold">{startItem}</span> to{" "}
      <span className="text-white font-bold">{endItem}</span> of{" "}
      <span className="text-white font-bold">{totalItems}</span>
    </div>

    {/* Pagination controls */}
    <div className="flex items-center gap-2 sm:gap-3">
      <button
        onClick={() => onPageChange(Math.max(1, currentPage - 1))}
        disabled={currentPage === 1}
        className="flex items-center gap-1 sm:gap-2 px-2 sm:px-3 lg:px-4 py-2 rounded-xl bg-[#1A1D29] border border-white/5 text-white/50 hover:text-white hover:border-white/20 disabled:opacity-20 disabled:cursor-not-allowed transition-all text-xs sm:text-sm font-medium"
      >
        <ChevronLeft size={14} className="sm:w-4 sm:h-4" />
        <span className="hidden sm:inline">Prev</span>
      </button>

      <div className="flex items-center gap-1 sm:gap-2">
        {pages.map((page) => (
          <button
            key={page}
            onClick={() => onPageChange(page)}
            className={`w-8 h-8 sm:w-9 sm:h-9 lg:w-10 lg:h-10 rounded-full flex items-center justify-center text-xs sm:text-sm font-semibold transition-all ${
              currentPage === page
                ? "bg-[#5B6FE8] text-white shadow-lg shadow-[#5B6FE8]/20"
                : "bg-[#1A1D29] border border-white/5 text-white hover:text-white hover:border-white/20"
            }`}
          >
            {page}
          </button>
        ))}
      </div>

      <button
        onClick={() => onPageChange(Math.min(totalPages, currentPage + 1))}
        disabled={currentPage === totalPages}
        className="flex items-center gap-1 sm:gap-2 px-2 sm:px-3 lg:px-4 py-2 rounded-xl bg-[#1A1D29] border border-white/5 text-white/50 hover:text-white hover:border-white/20 disabled:opacity-20 disabled:cursor-not-allowed transition-all text-xs sm:text-sm font-medium"
      >
        <span className="hidden sm:inline">Next</span>
        <ChevronRight size={14} className="sm:w-4 sm:h-4" />
      </button>
    </div>
  </div>
);
}
