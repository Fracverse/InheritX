"use client";

import React, { useState } from "react";
import { Plus, Search, Filter, History, Mail, AlertCircle } from "lucide-react";
import MessageCard from "./components/MessageCard";
import CreateMessageModal from "./components/CreateMessageModal";
import { LegacyMessage, MessageAuditLog } from "@/app/lib/api/messages";

const MOCK_MESSAGES: LegacyMessage[] = [
  {
    id: "msg_001",
    vault_id: "vault_001",
    title: "Key to Physical Safe",
    content_encrypted: "encrypted_data_here",
    unlock_at: "2026-01-01T00:00:00Z",
    status: "FINALIZED",
    created_at: "2024-03-15T10:00:00Z",
    updated_at: "2024-03-15T10:00:00Z",
    beneficiary_ids: ["ben_001", "ben_002"]
  },
  {
    id: "msg_002",
    vault_id: "vault_002",
    title: "Business Succession Plan",
    content_encrypted: "encrypted_data_here",
    unlock_at: "2025-12-25T00:00:00Z",
    status: "DRAFT",
    created_at: "2024-04-10T14:30:00Z",
    updated_at: "2024-04-12T09:15:00Z",
    beneficiary_ids: ["ben_003"]
  },
  {
    id: "msg_003",
    vault_id: "vault_001",
    title: "Welcome Message for Grandkids",
    content_encrypted: "encrypted_data_here",
    unlock_at: "2024-01-01T00:00:00Z",
    status: "UNLOCKED",
    created_at: "2023-12-01T08:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
    beneficiary_ids: ["ben_001", "ben_002", "ben_004"]
  }
];

const MOCK_AUDIT: MessageAuditLog[] = [
  { id: "a_1", message_id: "msg_001", action: "FINALIZE", performed_by: "Owner", timestamp: "2024-03-16T11:00:00Z" },
  { id: "a_2", message_id: "msg_003", action: "UNLOCK", performed_by: "ben_001", timestamp: "2024-01-02T10:15:00Z" },
  { id: "a_3", message_id: "msg_003", action: "ACCESS", performed_by: "ben_002", timestamp: "2024-01-03T09:00:00Z" }
];

export default function MessagesPage() {
  const [messages, setMessages] = useState<LegacyMessage[]>(MOCK_MESSAGES);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingMessage, setEditingMessage] = useState<LegacyMessage | null>(null);
  const [activeTab, setActiveTab] = useState<"all" | "drafts" | "finalized" | "unlocked">("all");
  const [showAudit, setShowAudit] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);

  React.useEffect(() => {
    setCurrentTime(Date.now());
  }, []);

  const filteredMessages = messages.filter(m => {
    if (activeTab === "all") return true;
    const statusLower = m.status.toLowerCase();
    const tabLower = activeTab.toLowerCase();
    // Match "drafts" with "draft", "finalized" with "finalized", etc.
    return statusLower === tabLower || statusLower === tabLower.replace(/s$/, '');
  });

  const handleSave = (data: {
    title: string;
    vault_id: string;
    content: string;
    unlock_at: string;
    beneficiaries: string;
    encrypt: boolean;
    beneficiary_ids: string[];
  }) => {
    if (editingMessage) {
      setMessages(messages.map(m => m.id === editingMessage.id ? { ...m, ...data, updated_at: new Date(currentTime).toISOString() } : m));
    } else {
      const newMessage: LegacyMessage = {
        id: `msg_${currentTime}`,
        ...data,
        status: 'DRAFT',
        created_at: new Date(currentTime).toISOString(),
        updated_at: new Date(currentTime).toISOString(),
        content_encrypted: "encrypted_" + btoa(data.content)
      };
      setMessages([newMessage, ...messages]);
    }
    setShowCreateModal(false);
    setEditingMessage(null);
  };

  const handleFinalize = (id: string) => {
    setMessages(messages.map(m => m.id === id ? { ...m, status: 'FINALIZED' } : m));
  };

  const handleDelete = (id: string) => {
    setMessages(messages.filter(m => m.id !== id));
  };

  return (
    <div className="space-y-8 max-w-6xl mx-auto">
      {/* Header */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold text-[#FCFFFF] mb-2 flex items-center gap-3">
            <Mail className="text-[#33C5E0]" size={32} />
            Legacy Messages
          </h1>
          <p className="text-[#92A5A8]">Compose secure messages to be delivered to your beneficiaries at a future date.</p>
        </div>
        <button 
          onClick={() => setShowCreateModal(true)}
          className="bg-[#33C5E0] text-[#161E22] px-6 py-3 rounded-full font-bold flex items-center gap-2 hover:bg-[#2AB8D3] transition-all hover:scale-105 active:scale-95 shadow-[0_0_20px_rgba(51,197,224,0.3)]"
        >
          <Plus size={20} />
          New Legacy Message
        </button>
      </div>

      {/* Toolbar */}
      <div className="flex flex-col md:flex-row justify-between items-center gap-4 bg-[#182024] p-2 rounded-2xl border border-[#1C252A]">
        <div className="flex bg-[#161E22] p-1 rounded-xl">
          {["all", "drafts", "finalized", "unlocked"].map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab as "all" | "drafts" | "finalized" | "unlocked")}
              className={`px-6 py-2 rounded-lg text-xs font-bold uppercase transition-all ${
                activeTab === tab ? 'bg-[#33C5E0] text-[#161E22]' : 'text-[#92A5A8] hover:text-[#FCFFFF]'
              }`}
            >
              {tab}
            </button>
          ))}
        </div>
        
        <div className="flex items-center gap-3 w-full md:w-auto">
          <div className="relative flex-1 md:w-64">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-[#92A5A8]" size={16} />
            <input 
              className="w-full bg-[#161E22] border border-[#2A3338] rounded-xl pl-10 pr-4 py-2 text-sm text-[#FCFFFF] outline-none focus:border-[#33C5E0] transition-colors" 
              placeholder="Search messages..." 
            />
          </div>
          <button className="p-2.5 bg-[#161E22] border border-[#2A3338] rounded-xl text-[#92A5A8] hover:text-[#FCFFFF] transition-colors">
            <Filter size={18} />
          </button>
          <button 
            onClick={() => setShowAudit(!showAudit)}
            className={`p-2.5 border rounded-xl transition-all ${showAudit ? 'bg-[#33C5E0]/10 border-[#33C5E0] text-[#33C5E0]' : 'bg-[#161E22] border-[#2A3338] text-[#92A5A8] hover:text-[#FCFFFF]'}`}
            title="View Audit Logs"
          >
            <History size={18} />
          </button>
        </div>
      </div>

      {/* Content */}
      {showAudit ? (
        <div className="bg-[#182024] rounded-2xl border border-[#1C252A] overflow-hidden animate-fade-in">
          <div className="p-6 border-b border-[#1C252A] bg-[#1C252A]/50 flex justify-between items-center">
            <h2 className="text-xl font-bold text-[#FCFFFF] flex items-center gap-2">
              <History size={20} className="text-[#33C5E0]" />
              Access Audit Trail
            </h2>
            <button onClick={() => setShowAudit(false)} className="text-xs text-[#33C5E0] hover:underline uppercase font-bold">Back to Messages</button>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-left">
              <thead>
                <tr className="text-[#92A5A8] text-[10px] uppercase font-bold tracking-widest border-b border-[#1C252A]">
                  <th className="px-8 py-4">Action</th>
                  <th className="px-8 py-4">Message ID</th>
                  <th className="px-8 py-4">User</th>
                  <th className="px-8 py-4">Timestamp</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[#1C252A]">
                {MOCK_AUDIT.map(log => (
                  <tr key={log.id} className="text-sm hover:bg-[#1C252A]/30 transition-colors">
                    <td className="px-8 py-4 text-[#FCFFFF] font-medium">{log.action}</td>
                    <td className="px-8 py-4 font-mono text-xs text-[#92A5A8]">{log.message_id}</td>
                    <td className="px-8 py-4 text-[#33C5E0]">{log.performed_by}</td>
                    <td className="px-8 py-4 text-[#92A5A8]">{new Date(log.timestamp).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {filteredMessages.map(msg => (
            <MessageCard 
              key={msg.id} 
              message={msg} 
              currentTime={currentTime}
              onEdit={(m) => {
                setEditingMessage(m);
                setShowCreateModal(true);
              }}
              onDelete={handleDelete}
              onFinalize={handleFinalize}
              onView={() => { /* Logic to open view modal */ }}
            />
          ))}

          {filteredMessages.length === 0 && (
            <div className="col-span-full py-20 text-center bg-[#182024]/50 rounded-[32px] border border-dashed border-[#1C252A]">
              <div className="inline-flex p-4 bg-[#1C252A] rounded-full text-[#33C5E0] mb-4">
                <AlertCircle size={32} />
              </div>
              <h3 className="text-xl font-bold text-[#FCFFFF] mb-2">No messages found</h3>
              <p className="text-[#92A5A8] max-w-sm mx-auto mb-8">
                Start by creating a new legacy message for your beneficiaries.
              </p>
              <button 
                onClick={() => setShowCreateModal(true)}
                className="bg-[#33C5E0] text-[#161E22] px-8 py-3 rounded-full font-bold hover:bg-[#2AB8D3] transition-all"
              >
                Create Your First Message
              </button>
            </div>
          )}
        </div>
      )}

      {showCreateModal && (
        <CreateMessageModal 
          onClose={() => {
            setShowCreateModal(false);
            setEditingMessage(null);
          }} 
          onSave={handleSave}
          initialData={editingMessage}
        />
      )}
    </div>
  );
}
