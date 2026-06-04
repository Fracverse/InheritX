"use client";

import React from "react";
import { Shield, Clock, AlertTriangle, ArrowRight } from "lucide-react";
import { EmergencyStatus as StatusType } from "@/app/lib/api/emergency";

interface Props {
  status: StatusType;
  currentTime: number;
  onActivate: () => void;
  onRevoke: () => void;
}

export default function EmergencyStatus({ status, currentTime, onActivate, onRevoke }: Props) {
  const isActive = status.is_active;
  const isCooldown = status.cooldown_until && new Date(status.cooldown_until).getTime() > currentTime;

  return (
    <div className="grid gap-6 md:grid-cols-2">
      <div className="bg-[#182024] rounded-2xl p-6 border border-[#1C252A]">
        <div className="flex items-start justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className={`p-3 rounded-xl ${isActive ? 'bg-red-500/10 text-red-500' : 'bg-[#33C5E0]/10 text-[#33C5E0]'}`}>
              <Shield size={24} />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-[#FCFFFF]">Access Status</h2>
              <p className="text-sm text-[#92A5A8]">
                {isActive ? 'Emergency Access is currently ACTIVE' : 'System is monitoring for triggers'}
              </p>
            </div>
          </div>
          <div className={`px-3 py-1 rounded-full text-xs font-medium ${isActive ? 'bg-red-500 text-white' : 'bg-[#1C252A] text-[#92A5A8]'}`}>
            {isActive ? 'ACTIVE' : 'INACTIVE'}
          </div>
        </div>

        {isActive ? (
          <div className="space-y-4">
            <div className="p-4 bg-red-500/5 rounded-xl border border-red-500/20">
              <div className="flex items-center gap-2 text-red-500 mb-2">
                <Clock size={16} />
                <span className="text-sm font-semibold uppercase">Expires In</span>
              </div>
              <p className="text-2xl font-bold text-[#FCFFFF]">6 Days 23:54:12</p>
              <p className="text-xs text-[#92A5A8] mt-1">Access automatically revokes after 7 days</p>
            </div>
            <button 
              onClick={onRevoke}
              className="w-full py-4 bg-red-500 hover:bg-red-600 text-white rounded-xl font-semibold transition-colors flex items-center justify-center gap-2"
            >
              REVOKE ACCESS NOW
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="p-4 bg-[#161E22] rounded-xl border border-[#1C252A]">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-[#92A5A8]">Withdrawal Limit</span>
                <span className="text-[#33C5E0] font-semibold">{status.withdrawal_limit_percent}%</span>
              </div>
              <div className="w-full bg-[#1C252A] h-2 rounded-full overflow-hidden">
                <div className="bg-[#33C5E0] h-full" style={{ width: `${status.withdrawal_limit_percent}%` }} />
              </div>
              <p className="text-[10px] text-[#92A5A8] mt-2 italic">
                *Limited to 10% of total assets during emergency periods
              </p>
            </div>
            {isCooldown && (
              <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-xl flex items-center gap-3">
                <AlertTriangle size={18} className="text-yellow-500" />
                <div className="text-xs text-yellow-500">
                  <p className="font-bold uppercase">24-Hour Cooldown</p>
                  <p>System is resetting. Full access restores in 14h 22m.</p>
                </div>
              </div>
            )}
            {!isCooldown && (
              <button 
                onClick={onActivate}
                className="w-full py-4 bg-transparent border border-[#33C5E0] text-[#33C5E0] hover:bg-[#33C5E0]/5 rounded-xl font-semibold transition-colors flex items-center justify-center gap-2"
              >
                MANUALLY ACTIVATE EMERGENCY
                <ArrowRight size={18} />
              </button>
            )}
          </div>
        )}
      </div>

      <div className="bg-[#182024] rounded-2xl p-6 border border-[#1C252A] flex flex-col justify-center items-center text-center space-y-4">
        <div className="w-16 h-16 bg-[#33C5E0]/10 rounded-full flex items-center justify-center text-[#33C5E0] mb-2">
          <AlertTriangle size={32} />
        </div>
        <h3 className="text-xl font-bold text-[#FCFFFF]">Emergency Protocol</h3>
        <p className="text-sm text-[#92A5A8] max-w-[300px]">
          In case of lost access, your trusted contacts can request emergency access which must be approved by your designated guardians.
        </p>
        <div className="flex gap-2 text-xs">
          <span className="px-2 py-1 bg-[#1C252A] text-[#FCFFFF] rounded">7 Day Duration</span>
          <span className="px-2 py-1 bg-[#1C252A] text-[#FCFFFF] rounded">Guardian Approval</span>
          <span className="px-2 py-1 bg-[#1C252A] text-[#FCFFFF] rounded">10% Withdrawal</span>
        </div>
      </div>
    </div>
  );
}
