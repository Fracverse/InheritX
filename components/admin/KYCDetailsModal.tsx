"use client";

import React from "react";
import { 
    X, 
    Download, 
    Search 
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { clsx, type ClassValue } from "clsx";
import Image from "next/image";

// Utility for tailwind classes
function cn(...inputs: ClassValue[]) {
  return clsx(inputs);
}

const StatusBadge = ({ status }: { status: string }) => {
    const variants = {
        APPROVED: "bg-[#48BB78]/10 text-[#48BB78] border-[#48BB78]/20",
        PENDING: "bg-[#ECC94B]/10 text-[#ECC94B] border-[#ECC94B]/20",
        REJECTED: "bg-[#F56565]/10 text-[#F56565] border-[#F56565]/20",
    };
    
    return (
        <span className={cn(
            "px-2.5 py-1 rounded text-[10px] font-bold tracking-wider border",
            variants[status as keyof typeof variants] || "bg-gray-500/10 text-gray-500 border-gray-500/20"
        )}>
            {status}
        </span>
    );
};

export interface KYCApplication {
    id: string;
    user: {
        name: string;
        email: string;
    };
    idType: string;
    submittedAt: string;
    status: string;
    details: {
        dob: string;
        nationality: string;
        walletAddress: string;
        idNumber: string;
        expiryDate: string;
        address: {
            street: string;
            city: string;
            country: string;
            postalCode: string;
        };
        documentImage: string;
    };
}

interface KYCDetailsModalProps {
    isOpen: boolean;
    onClose: () => void;
    application: KYCApplication | null;
}

export function KYCDetailsModal({ isOpen, onClose, application }: KYCDetailsModalProps) {
    if (!isOpen || !application) return null;

    return (
        <AnimatePresence>
            <motion.div 
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
                onClick={onClose}
            >
                <motion.div 
                    initial={{ scale: 0.95, opacity: 0, y: 20 }}
                    animate={{ scale: 1, opacity: 1, y: 0 }}
                    exit={{ scale: 0.95, opacity: 0, y: 20 }}
                    className="bg-[#0A0F11] border border-[#161E22] w-full max-w-2xl max-h-[90vh] overflow-hidden rounded-2xl shadow-2xl"
                    onClick={(e) => e.stopPropagation()}
                >
                    {/* Modal Header */}
                    <div className="flex items-center justify-between p-6 border-b border-[#161E22]">
                        <h2 className="text-xl font-bold text-white tracking-tight">KYC Application Details</h2>
                        <button 
                            onClick={onClose}
                            className="p-2 hover:bg-white/10 rounded-full transition-colors text-[#8899A6] hover:text-white"
                        >
                            <X size={20} />
                        </button>
                    </div>

                    {/* Modal Content */}
                    <div className="p-6 overflow-y-auto max-h-[calc(90vh-140px)] no-scrollbar">
                        <div className="space-y-8">
                            {/* Status Section */}
                            <div className="flex items-center justify-between px-2">
                                <span className="text-[#8899A6] text-sm font-medium">Status</span>
                                <StatusBadge status={application.status} />
                            </div>

                            {/* Personal Information */}
                            <section>
                                <h3 className="text-[#33C5E0] text-[11px] font-bold uppercase tracking-wider mb-3 px-1">Personal Information</h3>
                                <div className="bg-[#060B0D] border border-[#161E22] rounded-xl p-5 space-y-4">
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Full Name</span>
                                        <span className="text-white text-sm font-semibold">{application.user.name}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Email</span>
                                        <span className="text-white text-sm font-semibold">{application.user.email}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Date of Birth</span>
                                        <span className="text-white text-sm font-semibold">{application.details.dob}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Nationality</span>
                                        <span className="text-white text-sm font-semibold">{application.details.nationality}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Wallet Address</span>
                                        <span className="text-white text-sm font-mono text-[13px]">{application.details.walletAddress}</span>
                                    </div>
                                </div>
                            </section>

                            {/* ID Information */}
                            <section>
                                <h3 className="text-[#33C5E0] text-[11px] font-bold uppercase tracking-wider mb-3 px-1">ID Information</h3>
                                <div className="bg-[#060B0D] border border-[#161E22] rounded-xl p-5 space-y-4">
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">ID Type</span>
                                        <span className="text-white text-sm font-semibold uppercase">{application.idType}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">ID Number</span>
                                        <span className="text-white text-sm font-semibold">{application.details.idNumber}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Expiry Date</span>
                                        <span className="text-white text-sm font-semibold">{application.details.expiryDate}</span>
                                    </div>
                                </div>
                            </section>

                            {/* Address */}
                            <section>
                                <h3 className="text-[#33C5E0] text-[11px] font-bold uppercase tracking-wider mb-3 px-1">Address</h3>
                                <div className="bg-[#060B0D] border border-[#161E22] rounded-xl p-5 space-y-4">
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Street Address</span>
                                        <span className="text-white text-sm font-semibold">{application.details.address.street}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">City</span>
                                        <span className="text-white text-sm font-semibold">{application.details.address.city}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Country</span>
                                        <span className="text-white text-sm font-semibold">{application.details.address.country}</span>
                                    </div>
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Postal Code</span>
                                        <span className="text-white text-sm font-semibold">{application.details.address.postalCode}</span>
                                    </div>
                                </div>
                            </section>

                            {/* Submission Details */}
                            <section>
                                <h3 className="text-[#33C5E0] text-[11px] font-bold uppercase tracking-wider mb-3 px-1">Submission Details</h3>
                                <div className="bg-[#060B0D] border border-[#161E22] rounded-xl p-5">
                                    <div className="flex justify-between items-center group">
                                        <span className="text-[#8899A6] text-sm group-hover:text-[#33C5E0] transition-colors">Submitted At</span>
                                        <span className="text-white text-sm font-semibold">{application.submittedAt}</span>
                                    </div>
                                </div>
                            </section>

                            {/* ID Document */}
                            <section>
                                <h3 className="text-[#33C5E0] text-[11px] font-bold uppercase tracking-wider mb-3 px-1">ID Document</h3>
                                <div className="bg-[#060B0D] border border-[#161E22] rounded-xl p-2 relative aspect-[4/3] overflow-hidden group">
                                    <Image 
                                        src={application.details.documentImage} 
                                        alt="ID Document" 
                                        fill
                                        className="object-contain rounded-lg group-hover:scale-105 transition-transform duration-500"
                                    />
                                    <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                        <button className="bg-white/10 backdrop-blur-md border border-white/20 p-3 rounded-full text-white hover:bg-white/20">
                                            <Search size={24} />
                                        </button>
                                    </div>
                                </div>
                                <button className="mt-4 w-full flex items-center justify-center gap-2 bg-[#161E22] hover:bg-[#1C262B] text-white py-3.5 rounded-xl font-bold transition-all border border-[#2A353A] group">
                                    <Download size={18} className="group-hover:translate-y-0.5 transition-transform" />
                                    Open Full Size
                                </button>
                            </section>
                        </div>
                    </div>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
}
