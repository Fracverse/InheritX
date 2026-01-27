"use client";

import React, { useState } from "react";
import { 
    Search, 
    Eye, 
} from "lucide-react";
import { motion } from "framer-motion";
import { clsx, type ClassValue } from "clsx";
import { KYCDetailsModal, type KYCApplication } from "@/components/admin/KYCDetailsModal";

// Utility for tailwind classes
function cn(...inputs: ClassValue[]) {
  return clsx(inputs);
}

// --- Mock Data ---

const KYC_APPLICATIONS: KYCApplication[] = [
    {
        id: "1",
        user: {
            name: "Ebube Ebuka Onuora",
            email: "onuoraebube44@gmail.com"
        },
        idType: "PASSPORT",
        submittedAt: "Jan 18, 2026, 12:06 PM",
        status: "APPROVED",
        details: {
            dob: "05/12/1995",
            nationality: "Nigeria",
            walletAddress: "0x7a12...c421",
            idNumber: "A00123456",
            expiryDate: "12/05/2028",
            address: {
                street: "123 Lekki Phase 1",
                city: "Lagos",
                country: "Nigeria",
                postalCode: "101233"
            },
            documentImage: "https://images.unsplash.com/photo-1557683316-973673baf926?q=80&w=2029&auto=format&fit=crop"
        }
    },
    {
        id: "2",
        user: {
            name: "Adamu Jethro",
            email: "jethroadamzy@gmail.com"
        },
        idType: "PASSPORT",
        submittedAt: "Jan 17, 2026, 08:35 PM",
        status: "APPROVED",
        details: {
            dob: "10/01/1998",
            nationality: "Nigeria",
            walletAddress: "0x4ff8...b807",
            idNumber: "P98765432",
            expiryDate: "15/09/2027",
            address: {
                street: "No. 254 U/Zawu Gonin Gora",
                city: "Gonin Gora",
                country: "Nigeria",
                postalCode: "23654"
            },
            documentImage: "https://images.unsplash.com/photo-1633113088983-12fb3b2fe4ac?q=80&w=2070&auto=format&fit=crop"
        }
    },
    {
        id: "3",
        user: {
            name: "Adamu Jethro",
            email: "jethroadamzy@gmail.com"
        },
        idType: "DRIVERS LICENSE",
        submittedAt: "Jan 17, 2026, 07:38 PM",
        status: "APPROVED",
        details: {
            dob: "10/01/1998",
            nationality: "Nigeria",
            walletAddress: "0x4ff8...b807",
            idNumber: "45858493853",
            expiryDate: "11/02/1996",
            address: {
                street: "No. 254 U/Zawu Gonin Gora",
                city: "Gonin Gora",
                country: "Nigeria",
                postalCode: "23654"
            },
            documentImage: "https://images.unsplash.com/photo-1563986768609-322da13575f3?q=80&w=2070&auto=format&fit=crop"
        }
    }
];

const StatusBadge = ({ status }: { status: string }) => {
    const variants = {
        APPROVED: "bg-[#48BB78]/10 text-[#48BB78] border-[#48BB78]/20",
        PENDING: "bg-[#ECC94B]/10 text-[#ECC94B] border-[#ECC94B]/20",
        REJECTED: "bg-[#F56565]/10 text-[#F56565] border-[#F56565]/20",
    };
    
    return (
        <span className={cn(
            "px-2.5 py-1 rounded text-[10px] font-bold tracking-wider border whitespace-nowrap",
            variants[status as keyof typeof variants] || "bg-gray-500/10 text-gray-500 border-gray-500/20"
        )}>
            {status}
        </span>
    );
};

export default function KYCManagementPage() {
    const [searchTerm, setSearchTerm] = useState("");
    const [activeTab, setActiveTab] = useState("All");
    const [selectedApp, setSelectedApp] = useState<KYCApplication | null>(null);
    const [isModalOpen, setIsModalOpen] = useState(false);

    const tabs = ["All", "Pending", "Approved", "Rejected"];

    const filteredApplications = KYC_APPLICATIONS.filter(app => {
        const matchesSearch = app.user.name.toLowerCase().includes(searchTerm.toLowerCase()) || 
                             app.user.email.toLowerCase().includes(searchTerm.toLowerCase());
        const matchesTab = activeTab === "All" || app.status.toUpperCase() === activeTab.toUpperCase();
        return matchesSearch && matchesTab;
    });

    const handleViewDetails = (app: KYCApplication) => {
        setSelectedApp(app);
        setIsModalOpen(true);
    };

    return (
        <div className="w-full min-w-0">
            <div className="max-w-7xl mx-auto space-y-6 md:space-y-8 animate-fade-in  md:px-0">
                {/* Header */}
                <div>
                    <h1 className="text-2xl md:text-3xl font-bold text-white tracking-tight mb-2">KYC Management</h1>
                    <p className="text-[#8899A6] text-sm md:text-base font-medium">Review and manage user KYC applications.</p>
                </div>

                {/* Filters */}
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div className="relative flex-1 max-w-full lg:max-w-lg">
                        <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-[#4A5568]" size={18} />
                        <input 
                            type="text" 
                            placeholder="Search by name or email..."
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            className="w-full bg-[#0A0F11] border border-[#161E22] rounded-xl py-3.5 pl-12 pr-4 text-white placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0]/50 transition-all text-sm"
                        />
                    </div>

                    <div className="flex p-1 bg-[#0A0F11] border border-[#161E22] rounded-xl overflow-x-auto">
                        <div className="flex min-w-max">
                            {tabs.map((tab) => (
                                <button
                                    key={tab}
                                    onClick={() => setActiveTab(tab)}
                                    className={cn(
                                        "px-4 md:px-6 py-2 rounded-lg text-xs md:text-sm font-semibold transition-all duration-200 whitespace-nowrap",
                                        activeTab === tab 
                                            ? "bg-[#33C5E0] text-[#060B0D] shadow-lg shadow-[#33C5E0]/20" 
                                            : "text-[#8899A6] hover:text-white"
                                    )}
                                >
                                    {tab}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Table View */}
                <div className="w-full bg-[#0A0F11] border border-[#161E22] rounded-2xl shadow-xl overflow-hidden">
                    <div className="w-full overflow-x-auto">
                        <table className="w-full min-w-[800px] border-collapse">
                            <thead>
                                <tr className="border-b border-[#161E22] bg-[#0A0F11]/50">
                                    <th className="text-left py-5 px-4 md:px-6 text-[#4A5568] text-[10px] font-bold uppercase tracking-wider whitespace-nowrap">User</th>
                                    <th className="text-left py-5 px-4 md:px-6 text-[#4A5568] text-[10px] font-bold uppercase tracking-wider whitespace-nowrap">ID Type</th>
                                    <th className="text-left py-5 px-4 md:px-6 text-[#4A5568] text-[10px] font-bold uppercase tracking-wider whitespace-nowrap">Submitted</th>
                                    <th className="text-left py-5 px-4 md:px-6 text-[#4A5568] text-[10px] font-bold uppercase tracking-wider whitespace-nowrap">Status</th>
                                    <th className="text-right py-5 px-4 md:px-6 text-[#4A5568] text-[10px] font-bold uppercase tracking-wider whitespace-nowrap">Actions</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-[#161E22]">
                                {filteredApplications.length > 0 ? (
                                    filteredApplications.map((app, index) => (
                                        <motion.tr 
                                            key={app.id}
                                            initial={{ opacity: 0, y: 10 }}
                                            animate={{ opacity: 1, y: 0 }}
                                            transition={{ duration: 0.3, delay: index * 0.05 }}
                                            className="group hover:bg-[#161E22]/30 transition-colors cursor-pointer"
                                            onClick={() => handleViewDetails(app)}
                                        >
                                            <td className="py-5 px-4 md:px-6 whitespace-nowrap">
                                                <div className="flex flex-col">
                                                    <span className="text-white font-bold group-hover:text-[#33C5E0] transition-colors text-sm">{app.user.name}</span>
                                                    <span className="text-[#8899A6] text-xs font-medium">{app.user.email}</span>
                                                </div>
                                            </td>
                                            <td className="py-5 px-4 md:px-6 whitespace-nowrap">
                                                <span className="text-white text-xs md:text-sm font-semibold uppercase">{app.idType}</span>
                                            </td>
                                            <td className="py-5 px-4 md:px-6 whitespace-nowrap">
                                                <span className="text-[#8899A6] text-xs md:text-sm font-medium">{app.submittedAt}</span>
                                            </td>
                                            <td className="py-5 px-4 md:px-6 whitespace-nowrap">
                                                <StatusBadge status={app.status} />
                                            </td>
                                            <td className="py-5 px-4 md:px-6 text-right whitespace-nowrap">
                                                <button 
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        handleViewDetails(app);
                                                    }}
                                                    className="p-2 text-[#8899A6] hover:text-[#33C5E0] bg-[#161E22] hover:bg-[#33C5E0]/10 rounded-lg transition-all"
                                                >
                                                    <Eye size={18} />
                                                </button>
                                            </td>
                                        </motion.tr>
                                    ))
                                ) : (
                                    <tr>
                                        <td colSpan={5} className="py-20 text-center">
                                            <div className="flex flex-col items-center gap-3">
                                                <div className="p-4 bg-[#161E22] rounded-full text-[#4A5568]">
                                                    <Search size={32} />
                                                </div>
                                                <p className="text-[#8899A6] font-medium">No KYC applications found.</p>
                                            </div>
                                        </td>
                                    </tr>
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>

                {/* Modal Component */}
                <KYCDetailsModal 
                    isOpen={isModalOpen} 
                    onClose={() => setIsModalOpen(false)} 
                    application={selectedApp} 
                />
            </div>
        </div>
    );
}