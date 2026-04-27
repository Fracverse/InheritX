"use client";

import React, { useState } from "react";
import { UserPlus, Trash2, Mail, Wallet, Plus, X } from "lucide-react";
import { EmergencyContact } from "@/app/lib/api/emergency";

interface Props {
  contacts: EmergencyContact[];
  onAdd: (contact: Omit<EmergencyContact, 'id' | 'added_at'>) => void;
  onRemove: (id: string) => void;
}

export default function ContactManagement({ contacts, onAdd, onRemove }: Props) {
  const [isAdding, setIsAdding] = useState(false);
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    wallet_address: ""
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onAdd(formData);
    setFormData({ name: "", email: "", wallet_address: "" });
    setIsAdding(false);
  };

  const isLimitReached = contacts.length >= 10;

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-[#FCFFFF]">Emergency Contacts</h2>
          <p className="text-sm text-[#92A5A8]">
            Trusted individuals who can request access ({contacts.length}/10)
          </p>
        </div>
        {!isAdding && !isLimitReached && (
          <button
            onClick={() => setIsAdding(true)}
            className="flex items-center gap-2 bg-[#33C5E0] text-[#161E22] px-4 py-2 rounded-lg font-semibold hover:bg-[#2AB8D3] transition-colors"
          >
            <UserPlus size={18} />
            Add Contact
          </button>
        )}
      </div>

      {isAdding && (
        <form onSubmit={handleSubmit} className="bg-[#1C252A] p-6 rounded-2xl border border-[#33C5E0]/30 animate-fade-in">
          <div className="flex justify-between items-center mb-4">
            <h3 className="font-semibold text-[#FCFFFF]">New Emergency Contact</h3>
            <button type="button" onClick={() => setIsAdding(false)} className="text-[#92A5A8] hover:text-[#FCFFFF]">
              <X size={20} />
            </button>
          </div>
          <div className="grid gap-4 md:grid-cols-3 mb-6">
            <div className="space-y-2">
              <label className="text-xs text-[#92A5A8] uppercase font-bold">Full Name</label>
              <input
                required
                type="text"
                placeholder="John Doe"
                className="w-full bg-[#161E22] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs text-[#92A5A8] uppercase font-bold">Email Address</label>
              <input
                required
                type="email"
                placeholder="john@example.com"
                className="w-full bg-[#161E22] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs text-[#92A5A8] uppercase font-bold">Stellar Address</label>
              <input
                required
                type="text"
                placeholder="G..."
                className="w-full bg-[#161E22] border border-[#2A3338] rounded-xl px-4 py-3 text-[#FCFFFF] focus:border-[#33C5E0] outline-none transition-colors"
                value={formData.wallet_address}
                onChange={(e) => setFormData({ ...formData, wallet_address: e.target.value })}
              />
            </div>
          </div>
          <div className="flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setIsAdding(false)}
              className="px-6 py-2 text-[#92A5A8] hover:text-[#FCFFFF] transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-8 py-2 bg-[#33C5E0] text-[#161E22] rounded-lg font-bold hover:bg-[#2AB8D3] transition-colors"
            >
              Save Contact
            </button>
          </div>
        </form>
      )}

      <div className="grid gap-4 md:grid-cols-2">
        {contacts.map((contact) => (
          <div key={contact.id} className="bg-[#182024] p-5 rounded-2xl border border-[#1C252A] group hover:border-[#33C5E0]/20 transition-all">
            <div className="flex justify-between items-start mb-4">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-full bg-gradient-to-br from-[#33C5E0] to-[#8B5CF6] flex items-center justify-center text-white font-bold">
                  {contact.name.charAt(0).toUpperCase()}
                </div>
                <div>
                  <h4 className="font-semibold text-[#FCFFFF]">{contact.name}</h4>
                  <p className="text-xs text-[#92A5A8]">Added {new Date(contact.added_at).toLocaleDateString()}</p>
                </div>
              </div>
              <button
                onClick={() => onRemove(contact.id)}
                className="p-2 text-[#92A5A8] hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all"
              >
                <Trash2 size={18} />
              </button>
            </div>
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm text-[#92A5A8]">
                <Mail size={14} className="text-[#33C5E0]" />
                {contact.email}
              </div>
              <div className="flex items-center gap-2 text-sm text-[#92A5A8]">
                <Wallet size={14} className="text-[#33C5E0]" />
                <span className="truncate">{contact.wallet_address}</span>
              </div>
            </div>
          </div>
        ))}

        {contacts.length === 0 && !isAdding && (
          <div className="md:col-span-2 py-12 text-center bg-[#182024]/50 rounded-2xl border border-dashed border-[#1C252A]">
            <p className="text-[#92A5A8] mb-4">No emergency contacts added yet.</p>
            <button
              onClick={() => setIsAdding(true)}
              className="inline-flex items-center gap-2 text-[#33C5E0] hover:underline"
            >
              <Plus size={18} />
              Add your first trusted contact
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
