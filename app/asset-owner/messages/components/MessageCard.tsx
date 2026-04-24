"use client";

import React from "react";
import { Clock, Lock, Unlock, MoreVertical, Edit2, Trash2, ShieldCheck } from "lucide-react";
import { LegacyMessage } from "@/app/lib/api/messages";

interface Props {
  message: LegacyMessage;
  onEdit: (message: LegacyMessage) => void;
  onDelete: (id: string) => void;
  onFinalize: (id: string) => void;
  onView: (message: LegacyMessage) => void;
}

export default function MessageCard({ message, onEdit, onDelete, onFinalize, onView }: Props) {
  const isDraft = message.status === 'DRAFT';
  const isFinalized = message.status === 'FINALIZED';
  const isUnlocked = message.status === 'UNLOCKED';

  const timeRemaining = () => {
    const diff = new Date(message.unlock_at).getTime() - Date.now();
    if (diff <= 0) return "Ready to unlock";
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    return `${days}d ${hours}h remaining`;
  };

  return (
    <div className="bg-[#182024] rounded-2xl p-6 border border-[#1C252A] group hover:border-[#33C5E0]/30 transition-all">
      <div className="flex justify-between items-start mb-4">
        <div className="flex items-center gap-3">
          <div className={`p-2.5 rounded-xl ${
            isUnlocked ? 'bg-green-500/10 text-green-500' : 
            isFinalized ? 'bg-[#33C5E0]/10 text-[#33C5E0]' : 
            'bg-gray-500/10 text-gray-500'
          }`}>
            {isUnlocked ? <Unlock size={20} /> : <Lock size={20} />}
          </div>
          <div>
            <h3 className="text-lg font-bold text-[#FCFFFF] group-hover:text-[#33C5E0] transition-colors">
              {message.title}
            </h3>
            <p className="text-xs text-[#92A5A8]">Vault: {message.vault_id}</p>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <span className={`px-2 py-1 rounded-md text-[10px] font-bold uppercase tracking-wider ${
            isUnlocked ? 'bg-green-500/10 text-green-500' : 
            isFinalized ? 'bg-[#33C5E0]/10 text-[#33C5E0]' : 
            'bg-[#1C252A] text-[#92A5A8]'
          }`}>
            {message.status}
          </span>
        </div>
      </div>

      <div className="space-y-4">
        <div className="flex items-center gap-2 text-sm text-[#92A5A8]">
          <Clock size={14} className="text-[#33C5E0]" />
          <span>Unlock Date: {new Date(message.unlock_at).toLocaleDateString()}</span>
          {!isUnlocked && (
            <span className="ml-auto text-[10px] font-mono bg-[#1C252A] px-2 py-0.5 rounded text-[#33C5E0]">
              {timeRemaining()}
            </span>
          )}
        </div>

        <div className="pt-4 border-t border-[#1C252A] flex items-center justify-between">
          <div className="flex -space-x-2">
            {message.beneficiary_ids.map((id, i) => (
              <div key={id} className="w-6 h-6 rounded-full bg-[#1C252A] border border-[#2A3338] flex items-center justify-center text-[10px] text-[#33C5E0]" title={`Beneficiary ${id}`}>
                {String.fromCharCode(65 + i)}
              </div>
            ))}
          </div>

          <div className="flex gap-2">
            {isDraft && (
              <>
                <button 
                  onClick={() => onEdit(message)}
                  className="p-2 text-[#92A5A8] hover:text-[#33C5E0] hover:bg-[#33C5E0]/10 rounded-lg transition-all"
                  title="Edit Message"
                >
                  <Edit2 size={16} />
                </button>
                <button 
                  onClick={() => onFinalize(message.id)}
                  className="p-2 text-[#92A5A8] hover:text-green-500 hover:bg-green-500/10 rounded-lg transition-all"
                  title="Finalize Message"
                >
                  <ShieldCheck size={16} />
                </button>
              </>
            )}
            {!isUnlocked && (
              <button 
                onClick={() => onDelete(message.id)}
                className="p-2 text-[#92A5A8] hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all"
                title="Delete Message"
              >
                <Trash2 size={16} />
              </button>
            )}
            <button 
              onClick={() => onView(message)}
              className="px-4 py-1.5 bg-[#1C252A] text-[#FCFFFF] text-xs font-bold rounded-lg hover:bg-[#2A3338] transition-all"
            >
              VIEW
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
