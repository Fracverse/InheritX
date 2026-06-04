"use client";

import React from "react";
import { PlusCircle, Settings, ShieldAlert, Download } from "lucide-react";

const actions = [
    { label: "New System Alert", icon: ShieldAlert },
    { label: "Platform Settings", icon: Settings },
    { label: "Export Logs", icon: Download },
    { label: "Add Admin", icon: PlusCircle },
];

export function QuickActions() {
    return (
        <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-6">
            <h3 className="font-bold text-white mb-6">Quick Actions</h3>
            <div className="grid grid-cols-2 gap-3">
                {actions.map((action) => (
                    <button
                        key={action.label}
                        className="flex flex-col items-center justify-center gap-2 p-4 bg-[#161E22] border border-[#161E22] rounded-xl text-[#8899A6] hover:text-[#33C5E0] hover:border-[#33C5E0]/30 hover:bg-[#33C5E0]/5 transition-all duration-200"
                    >
                        <action.icon size={20} />
                        <span className="text-[11px] font-bold text-center leading-tight">{action.label}</span>
                    </button>
                ))}
            </div>
        </div>
    );
}
