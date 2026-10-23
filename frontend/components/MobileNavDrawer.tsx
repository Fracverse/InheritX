"use client";

import React, { useState, useCallback, useRef, useEffect } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion, AnimatePresence, useMotionValue } from "framer-motion";
import { X, Menu, ChevronRight } from "lucide-react";

interface NavItem {
  label: string;
  href: string;
  icon: React.ReactNode;
}

interface MobileNavDrawerProps {
  items: NavItem[];
  isOpen: boolean;
  onClose: () => void;
}

const SWIPE_THRESHOLD = 80;
const MIN_TAP_TARGET = 44;

export function MobileNavDrawer({ items, isOpen, onClose }: MobileNavDrawerProps) {
  const pathname = usePathname();
  const drawerRef = useRef<HTMLDivElement>(null);
  const startXRef = useRef(0);
  const startYRef = useRef(0);
  const isDraggingRef = useRef(false);
  const [activeIndex, setActiveIndex] = useState(-1);

  const x = useMotionValue(0);

  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    startXRef.current = e.touches[0].clientX;
    startYRef.current = e.touches[0].clientY;
    isDraggingRef.current = true;
  }, []);

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!isDraggingRef.current) return;
      const currentX = e.touches[0].clientX;
      const currentY = e.touches[0].clientY;
      const deltaX = currentX - startXRef.current;
      const deltaY = currentY - startYRef.current;

      if (Math.abs(deltaX) > Math.abs(deltaY)) {
        if (deltaX > 0) {
          x.set(deltaX);
        }
      }
    },
    [x]
  );

  const handleTouchEnd = useCallback(() => {
    if (!isDraggingRef.current) return;
    isDraggingRef.current = false;
    const currentX = x.get();
    if (currentX > SWIPE_THRESHOLD) {
      onClose();
    } else {
      x.set(0);
    }
  }, [x, onClose]);

  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = "hidden";
      drawerRef.current?.focus();
    } else {
      document.body.style.overflow = "";
      x.set(0);
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [isOpen, x]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
        return;
      }
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        const direction = e.key === "ArrowDown" ? 1 : -1;
        setActiveIndex((prev) => {
          const next = Math.max(0, Math.min(items.length - 1, prev + direction));
          const links = drawerRef.current?.querySelectorAll<HTMLAnchorElement>("a[role='menuitem']");
          links?.[next]?.focus();
          return next;
        });
      }
    },
    [items.length, onClose]
  );

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
            onClick={onClose}
            aria-hidden="true"
          />

          {/* Drawer */}
          <motion.div
            ref={drawerRef}
            role="dialog"
            aria-modal="true"
            aria-label="Navigation menu"
            tabIndex={-1}
            initial={{ x: "-100%" }}
            animate={{ x: 0 }}
            exit={{ x: "-100%" }}
            transition={{ type: "spring", damping: 30, stiffness: 300 }}
            style={{ x }}
            className="fixed inset-y-0 left-0 z-50 w-80 max-w-[85vw] bg-[#0A0F11] border-r border-white/[0.06] overflow-y-auto focus:outline-none"
            onTouchStart={handleTouchStart}
            onTouchMove={handleTouchMove}
            onTouchEnd={handleTouchEnd}
            onKeyDown={handleKeyDown}
            data-testid="mobile-nav-drawer"
          >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-white/[0.06]">
              <span className="text-lg font-semibold text-white">Menu</span>
              <button
                onClick={onClose}
                className="p-2 -mr-2 text-gray-400 hover:text-white transition-colors"
                style={{ minWidth: MIN_TAP_TARGET, minHeight: MIN_TAP_TARGET }}
                aria-label="Close navigation menu"
              >
                <X size={20} />
              </button>
            </div>

            {/* Nav items */}
            <nav role="menu" aria-label="Navigation" className="p-3">
              {items.map((item, i) => {
                const isActive = pathname === item.href;
                return (
                  <Link
                    key={item.href}
                    href={item.href}
                    role="menuitem"
                    onClick={onClose}
                    tabIndex={i === 0 || i === activeIndex ? 0 : -1}
                    className={`flex items-center justify-between gap-3 rounded-xl transition-colors ${
                      isActive
                        ? "bg-[#33C5E0]/10 text-[#33C5E0] border border-[#33C5E0]/20"
                        : "text-gray-400 hover:text-white hover:bg-white/5 border border-transparent"
                    }`}
                    style={{ minHeight: MIN_TAP_TARGET }}
                    aria-current={isActive ? "page" : undefined}
                  >
                    <div className="flex items-center gap-3 px-4">
                      <span className={isActive ? "text-[#33C5E0]" : "text-gray-500"}>
                        {item.icon}
                      </span>
                      <span className="text-sm font-medium">{item.label}</span>
                    </div>
                    <ChevronRight
                      size={14}
                      className="mr-3 opacity-40"
                      aria-hidden="true"
                    />
                  </Link>
                );
              })}
            </nav>

            {/* Swipe hint */}
            <div className="p-4 mt-auto">
              <p className="text-[10px] text-gray-600 text-center">
                Swipe left to close
              </p>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}

export function MobileMenuButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="lg:hidden p-2 text-gray-400 hover:text-white transition-colors"
      style={{ minWidth: MIN_TAP_TARGET, minHeight: MIN_TAP_TARGET }}
      aria-label="Open navigation menu"
      aria-expanded="false"
      data-testid="mobile-menu-btn"
    >
      <Menu size={22} />
    </button>
  );
}
