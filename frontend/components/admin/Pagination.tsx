"use client";
import { PaginationProps } from "@/lib/adminTypes";

export function Pagination({ currentPage, totalPages, onPageChange }: PaginationProps) {
  if (totalPages <= 1) return null;

  const pages = Array.from({ length: totalPages }, (_, i) => i + 1);

  return (
    <div className="flex items-center justify-between mt-6">
      <p className="text-sm text-gray-500">
        Page {currentPage} of {totalPages}
      </p>

      <div className="flex items-center gap-1">
        {/* Previous */}
        <button
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage === 1}
          className="px-3 py-1.5 text-sm rounded-md border border-white/10 text-gray-400 hover:border-primary/50 hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          ← Prev
        </button>

        {/* Page numbers */}
        {pages.map((page) => (
          <button
            key={page}
            onClick={() => onPageChange(page)}
            className={`w-8 h-8 text-sm rounded-md border transition-colors ${
              page === currentPage
                ? "border-primary bg-primary/10 text-primary font-medium"
                : "border-white/10 text-gray-400 hover:border-primary/50 hover:text-primary"
            }`}
          >
            {page}
          </button>
        ))}

        {/* Next */}
        <button
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage === totalPages}
          className="px-3 py-1.5 text-sm rounded-md border border-white/10 text-gray-400 hover:border-primary/50 hover:text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          Next →
        </button>
      </div>
    </div>
  );
}
