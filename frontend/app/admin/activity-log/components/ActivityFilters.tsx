"use client";

import React, { useState, useRef, useEffect } from "react";
import { ChevronDown, Check } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

interface ActivityFiltersProps {
    tabs: string[];
    activeTab: string;
    onTabChange: (tab: string) => void;
}

export function ActivityFilters({ tabs, activeTab, onTabChange }: ActivityFiltersProps) {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        }
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    const handleSelect = (tab: string) => {
        onTabChange(tab);
        setIsOpen(false);
    };

    return (
        <div className="w-full relative z-20">
            <div className="sm:hidden" ref={dropdownRef}>
                <button
                    onClick={() => setIsOpen(!isOpen)}
                    className="w-full flex items-center justify-between bg-[#0A0F11] border border-[#161E22] text-white text-sm font-medium px-4 py-3 rounded-xl focus:outline-none focus:border-[#33C5E0] transition-colors"
                >
                    <span className="truncate">{activeTab}</span>
                    <ChevronDown size={16} className={`text-[#8899A6] transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`} />
                </button>

                <AnimatePresence>
                    {isOpen && (
                        <motion.div
                            initial={{ opacity: 0, y: -10 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: -10 }}
                            transition={{ duration: 0.2 }}
                            className="absolute top-full left-0 right-0 mt-2 bg-[#0A0F11] border border-[#161E22] rounded-xl shadow-xl overflow-hidden py-1"
                        >
                            {tabs.map((tab) => (
                                <button
                                    key={tab}
                                    onClick={() => handleSelect(tab)}
                                    className="w-full flex items-center justify-between px-4 py-3 text-left text-sm hover:bg-[#161E22] transition-colors"
                                >
                                    <span className={tab === activeTab ? "text-[#33C5E0] font-semibold" : "text-[#8899A6]"}>
                                        {tab}
                                    </span>
                                    {tab === activeTab && <Check size={14} className="text-[#33C5E0]" />}
                                </button>
                            ))}
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>

            <div className="hidden sm:flex flex-wrap gap-2">
                {tabs.map((tab) => (
                    <button
                        key={tab}
                        onClick={() => onTabChange(tab)}
                        className={`text-xs font-bold px-4 py-2.5 rounded-lg border uppercase tracking-wide transition-all duration-200 whitespace-nowrap ${
                            activeTab === tab
                                ? "bg-[#33C5E0] text-[#060B0D] border-[#33C5E0]"
                                : "bg-[#0A0F11] text-[#4A5568] border-[#161E22] hover:bg-[#161E22] hover:text-[#8899A6]"
                        }`}
                    >
                        {tab}
                    </button>
                ))}
            </div>
        </div>
    );
}
