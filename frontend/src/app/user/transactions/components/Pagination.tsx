export default function Pagination({
  currentPage,
  totalPages,
  onPageChange,
}: {
  currentPage: number;
  totalPages: number;
  totalItems?: number;
  itemsPerPage?: number;
  onPageChange: (p: number) => void;
}) {
  return (
    <div className="flex items-center justify-center gap-3 py-3">
      <button
        onClick={() => onPageChange(Math.max(1, currentPage - 1))}
        className="px-3 py-1 rounded bg-white/5"
        aria-label="Previous page"
      >
        Prev
      </button>
      <div className="text-sm">
        Page {currentPage} of {totalPages}
      </div>
      <button
        onClick={() => onPageChange(Math.min(totalPages, currentPage + 1))}
        className="px-3 py-1 rounded bg-white/5"
        aria-label="Next page"
      >
        Next
      </button>
    </div>
  );
}
