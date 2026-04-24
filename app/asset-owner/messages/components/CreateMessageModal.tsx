"use client";

import React, { useState } from "react";
import { X, Calendar, Shield, FileText, UserPlus } from "lucide-react";

interface Props {
  onClose: () => void;
  onSave: (data: {
    title: string;
    vault_id: string;
    content: string;
    unlock_at: string;
    beneficiaries: string;
    encrypt: boolean;
    beneficiary_ids: string[];
  }) => void;
  initialData?: Partial<LegacyMessage>;
}

export default function CreateMessageModal({ onClose, onSave, initialData }: Props) {
  const [formData, setFormData] = useState({
    title: initialData?.title || "",
    vault_id: initialData?.vault_id || "",
    content: initialData?.content || "",
    unlock_at: initialData?.unlock_at ? new Date(initialData.unlock_at).toISOString().split('T')[0] : "",
    beneficiaries: initialData?.beneficiary_ids?.join(", ") || "",
    encrypt: true
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      ...formData,
      beneficiary_ids: formData.beneficiaries.split(",").map((b: string) => b.trim()).filter((b: string) => b)
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
      <div className="bg-[#161E22] w-full max-w-2xl rounded-[32px] border border-[#2A3338] shadow-2xl overflow-hidden animate-fade-in">
        <div className="p-8 border-b border-[#2A3338] flex justify-between items-center bg-[#182024]">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-[#33C5E0]/10 text-[#33C5E0] rounded-lg">
              <FileText size={24} />
            </div>
            <div>
              <h2 className="text-2xl font-bold text-[#FCFFFF]">
                {initialData ? 'Edit Legacy Message' : 'Create Legacy Message'}
              </h2>
              <p className="text-sm text-[#92A5A8]">These messages will only be revealed at the specified time.</p>
            </div>
          </div>
          <button onClick={onClose} className="p-2 text-[#92A5A8] hover:text-[#FCFFFF] transition-colors">
            <X size={24} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-8 space-y-6 max-h-[70vh] overflow-y-auto scrollbar-hide">
          <div className="grid gap-6 md:grid-cols-2">
            <div className="space-y-2">
              <label className="text-xs font-bold text-[#92A5A8] uppercase tracking-wider">Message Title</label>
              <input
                required
                className="w-full bg-[#1C252A] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors"
                placeholder="e.g., Final Instructions for Family"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-bold text-[#92A5A8] uppercase tracking-wider">Select Vault</label>
              <select
                className="w-full bg-[#1C252A] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors"
                value={formData.vault_id}
                onChange={(e) => setFormData({ ...formData, vault_id: e.target.value })}
              >
                <option value="">Select a Vault</option>
                <option value="vault_001">Primary Family Vault</option>
                <option value="vault_002">Business Assets Vault</option>
              </select>
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-xs font-bold text-[#92A5A8] uppercase tracking-wider">Message Content</label>
            <textarea
              required
              rows={6}
              className="w-full bg-[#1C252A] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors resize-none"
              placeholder="Write your secure message here..."
              value={formData.content}
              onChange={(e) => setFormData({ ...formData, content: e.target.value })}
            />
          </div>

          <div className="grid gap-6 md:grid-cols-2">
            <div className="space-y-2">
              <label className="text-xs font-bold text-[#92A5A8] uppercase tracking-wider flex items-center gap-2">
                <Calendar size={14} />
                Unlock Date
              </label>
              <input
                required
                type="date"
                className="w-full bg-[#1C252A] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors"
                value={formData.unlock_at}
                onChange={(e) => setFormData({ ...formData, unlock_at: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-bold text-[#92A5A8] uppercase tracking-wider flex items-center gap-2">
                <UserPlus size={14} />
                Beneficiary IDs
              </label>
              <input
                className="w-full bg-[#1C252A] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors"
                placeholder="ID1, ID2, ID3"
                value={formData.beneficiaries}
                onChange={(e) => setFormData({ ...formData, beneficiaries: e.target.value })}
              />
            </div>
          </div>

          <div className="flex items-center gap-4 p-4 bg-[#33C5E0]/5 rounded-2xl border border-[#33C5E0]/20">
            <div className="p-2 bg-[#33C5E0] text-[#161E22] rounded-full">
              <Shield size={20} />
            </div>
            <div className="flex-1">
              <h4 className="text-sm font-bold text-[#FCFFFF]">End-to-End Encryption</h4>
              <p className="text-xs text-[#92A5A8]">Your message is encrypted client-side. Only authorized beneficiaries can decrypt it after the unlock date.</p>
            </div>
            <div className="relative inline-flex items-center cursor-pointer">
              <input 
                type="checkbox" 
                className="sr-only peer" 
                checked={formData.encrypt}
                onChange={() => setFormData({ ...formData, encrypt: !formData.encrypt })}
              />
              <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#33C5E0]"></div>
            </div>
          </div>

          <div className="flex gap-4 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 py-4 text-[#92A5A8] font-bold hover:bg-[#1C252A] rounded-2xl transition-colors"
            >
              Discard
            </button>
            <button
              type="submit"
              className="flex-1 py-4 bg-[#33C5E0] text-[#161E22] font-bold rounded-2xl hover:bg-[#2AB8D3] transition-all shadow-[0_0_20px_rgba(51,197,224,0.3)] hover:scale-[1.02] active:scale-[0.98]"
            >
              {initialData ? 'Save Changes' : 'Create & Encrypt'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
