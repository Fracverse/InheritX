"use client";

import React, { useState } from "react";
import { ShieldCheck, UserPlus, Trash2, Settings, AlertCircle } from "lucide-react";
import { Guardian } from "@/app/lib/api/emergency";

interface Props {
  guardians: Guardian[];
  threshold: number;
  onUpdateGuardians: (guardians: string[], threshold: number) => void;
}

export default function GuardianConfig({ guardians, threshold, onUpdateGuardians }: Props) {
  const [isEditing, setIsEditing] = useState(false);
  const [newGuardianAddr, setNewGuardianAddr] = useState("");
  const [localThreshold, setLocalThreshold] = useState(threshold);

  const handleSave = () => {
    const addresses = guardians.map(g => g.wallet_address);
    onUpdateGuardians(addresses, localThreshold);
    setIsEditing(false);
  };

  const removeGuardian = (addr: string) => {
    const updated = guardians.filter(g => g.wallet_address !== addr).map(g => g.wallet_address);
    // Ensure threshold doesn't exceed new count
    const newThreshold = Math.min(localThreshold, updated.length || 1);
    onUpdateGuardians(updated, newThreshold);
  };

  const addGuardian = () => {
    if (!newGuardianAddr) return;
    const updated = [...guardians.map(g => g.wallet_address), newGuardianAddr];
    onUpdateGuardians(updated, localThreshold);
    setNewGuardianAddr("");
  };

  return (
    <div className="bg-[#182024] rounded-2xl p-6 border border-[#1C252A]">
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-[#33C5E0]/10 text-[#33C5E0] rounded-lg">
            <ShieldCheck size={20} />
          </div>
          <h2 className="text-xl font-bold text-[#FCFFFF]">Guardians</h2>
        </div>
        {!isEditing && (
          <button
            onClick={() => setIsEditing(true)}
            className="text-sm text-[#33C5E0] hover:underline flex items-center gap-1"
          >
            <Settings size={14} />
            Configure Protocol
          </button>
        )}
      </div>

      <div className="space-y-6">
        <div className="p-4 bg-[#161E22] rounded-xl border border-[#1C252A]">
          <div className="flex justify-between items-center mb-2">
            <label className="text-sm text-[#92A5A8]">Approval Threshold</label>
            <span className="text-[#FCFFFF] font-bold">{threshold} of {guardians.length}</span>
          </div>
          <div className="flex gap-2">
            {[1, 2, 3, 4, 5].map((num) => (
              <button
                key={num}
                disabled={!isEditing || num > guardians.length}
                onClick={() => setLocalThreshold(num)}
                className={`flex-1 py-2 rounded-lg text-sm font-bold transition-all ${
                  localThreshold === num
                    ? 'bg-[#33C5E0] text-[#161E22]'
                    : 'bg-[#1C252A] text-[#92A5A8] hover:bg-[#2A3338]'
                } ${num > guardians.length ? 'opacity-20 cursor-not-allowed' : ''}`}
              >
                {num}
              </button>
            ))}
          </div>
          <p className="text-[10px] text-[#92A5A8] mt-3 flex items-center gap-1">
            <AlertCircle size={12} />
            Minimum number of guardians required to approve an access request.
          </p>
        </div>

        <div className="space-y-3">
          <label className="text-sm text-[#92A5A8]">Designated Guardians</label>
          <div className="space-y-2">
            {guardians.map((guardian) => (
              <div key={guardian.wallet_address} className="flex items-center justify-between p-3 bg-[#1C252A] rounded-xl border border-[#2A3338]">
                <div className="flex items-center gap-3">
                  <div className={`w-2 h-2 rounded-full ${guardian.is_approved ? 'bg-green-500' : 'bg-yellow-500'}`} />
                  <div>
                    <p className="text-sm text-[#FCFFFF] font-medium">{guardian.name || 'External Guardian'}</p>
                    <p className="text-[10px] text-[#92A5A8] truncate max-w-[200px]">{guardian.wallet_address}</p>
                  </div>
                </div>
                {isEditing && (
                  <button
                    onClick={() => removeGuardian(guardian.wallet_address)}
                    className="p-1.5 text-[#92A5A8] hover:text-red-500"
                  >
                    <Trash2 size={16} />
                  </button>
                )}
              </div>
            ))}
            
            {isEditing && (
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="Guardian Wallet Address"
                  className="flex-1 bg-[#161E22] border border-[#2A3338] rounded-lg px-4 py-2 text-sm text-[#FCFFFF] outline-none focus:border-[#33C5E0]"
                  value={newGuardianAddr}
                  onChange={(e) => setNewGuardianAddr(e.target.value)}
                />
                <button
                  onClick={addGuardian}
                  className="p-2 bg-[#33C5E0]/10 text-[#33C5E0] border border-[#33C5E0]/30 rounded-lg hover:bg-[#33C5E0]/20"
                >
                  <UserPlus size={18} />
                </button>
              </div>
            )}
          </div>
        </div>

        {isEditing && (
          <div className="flex gap-3 pt-2">
            <button
              onClick={() => {
                setIsEditing(false);
                setLocalThreshold(threshold);
              }}
              className="flex-1 py-3 text-[#92A5A8] hover:bg-[#1C252A] rounded-xl transition-colors font-semibold"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="flex-1 py-3 bg-[#33C5E0] text-[#161E22] rounded-xl hover:bg-[#2AB8D3] transition-colors font-bold"
            >
              Apply Changes
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
