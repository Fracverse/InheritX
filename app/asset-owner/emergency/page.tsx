"use client";

import React, { useState, useEffect } from "react";
import EmergencyStatus from "./components/EmergencyStatus";
import ContactManagement from "./components/ContactManagement";
import GuardianConfig from "./components/GuardianConfig";
import RequestInterface from "./components/RequestInterface";
import AuditLog from "./components/AuditLog";
import { 
  EmergencyContact, 
  Guardian, 
  EmergencyRequest, 
  AuditLog as LogType, 
  EmergencyStatus as StatusType 
} from "@/app/lib/api/emergency";

// Mock Data
const MOCK_CONTACTS: EmergencyContact[] = [
  {
    id: "1",
    name: "Jane Smith",
    email: "jane.smith@example.com",
    wallet_address: "GC2BK...X4Z2V",
    added_at: "2024-03-20T10:00:00Z"
  }
];

const MOCK_GUARDIANS: Guardian[] = [
  { id: "1", name: "Alice Security", wallet_address: "GD5FT...A3K9M", is_approved: true },
  { id: "2", name: "Bob Trust", wallet_address: "GA7L2...P1Q4W", is_approved: true },
  { id: "3", name: "Charlie Watch", wallet_address: "GB4U9...R8S2J", is_approved: false }
];

const MOCK_STATUS: StatusType = {
  is_active: false,
  withdrawal_limit_percent: 10,
  cooldown_until: "2024-04-25T17:00:00Z"
};

const MOCK_REQUESTS: EmergencyRequest[] = [
  {
    id: "req_1",
    requester_address: "GC2BK...X4Z2V",
    requester_name: "Jane Smith",
    plan_id: "plan_001",
    status: "PENDING",
    created_at: "2024-04-24T14:30:00Z",
    approvals_count: 1,
    threshold_required: 2
  }
];

const MOCK_LOGS: LogType[] = [
  { id: "log_1", action: "ADD_CONTACT", performed_by: "Owner (You)", details: "Added Jane Smith", timestamp: "2024-03-20T10:00:00Z" },
  { id: "log_2", action: "SET_GUARDIANS", performed_by: "Owner (You)", details: "Configured 3 guardians, threshold 2", timestamp: "2024-03-21T15:30:00Z" }
];

export default function EmergencyPage() {
  const [activeTab, setActiveTab] = useState<"overview" | "contacts" | "guardians" | "requests" | "logs">("overview");
  const [currentTime, setCurrentTime] = useState(0);

  useEffect(() => {
    setCurrentTime(Date.now());
  }, []);
  
  // Real state would come from API, using mock for now
  const [contacts, setContacts] = useState<EmergencyContact[]>(MOCK_CONTACTS);
  const [guardians, setGuardians] = useState<Guardian[]>(MOCK_GUARDIANS);
  const [status, setStatus] = useState<StatusType>(MOCK_STATUS);
  const [requests, setRequests] = useState<EmergencyRequest[]>(MOCK_REQUESTS);
  const [logs, setLogs] = useState<LogType[]>(MOCK_LOGS);

  const handleActivate = () => {
    setStatus({ ...status, is_active: true, activated_at: new Date(currentTime).toISOString() });
    addLog("ACTIVATE_EMERGENCY", "Owner (You)", "Manual activation");
  };

  const handleRevoke = () => {
    setStatus({ ...status, is_active: false, cooldown_until: new Date(currentTime + 86400000).toISOString() });
    addLog("REVOKE_EMERGENCY", "Owner (You)", "Manual revocation");
  };

  const addLog = (action: string, by: string, details: string) => {
    const newLog: LogType = {
      id: `log_${Date.now()}`,
      action,
      performed_by: by,
      details,
      timestamp: new Date().toISOString()
    };
    setLogs([newLog, ...logs]);
  };

  return (
    <div className="max-w-6xl mx-auto space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-[#FCFFFF] mb-2">Emergency Access</h1>
        <p className="text-[#92A5A8]">Manage trusted contacts, guardians, and emergency protocols for your assets.</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-[#182024] p-1 rounded-xl w-fit border border-[#1C252A]">
        {[
          { id: "overview", label: "Overview" },
          { id: "contacts", label: "Contacts" },
          { id: "guardians", label: "Guardians" },
          { id: "requests", label: "Requests" },
          { id: "logs", label: "Audit Logs" }
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as "overview" | "contacts" | "guardians" | "requests" | "logs")}
            className={`px-6 py-2.5 rounded-lg text-sm font-semibold transition-all ${
              activeTab === tab.id 
                ? 'bg-[#33C5E0] text-[#161E22] shadow-lg' 
                : 'text-[#92A5A8] hover:text-[#FCFFFF] hover:bg-[#1C252A]'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="animate-fade-in">
        {activeTab === "overview" && (
          <EmergencyStatus 
            status={status} 
            currentTime={currentTime}
            onActivate={handleActivate} 
            onRevoke={handleRevoke} 
          />
        )}

        {activeTab === "contacts" && (
          <ContactManagement 
            contacts={contacts}
            onAdd={(c: Omit<EmergencyContact, 'id' | 'added_at'>) => {
              const newContact = { ...c, id: Date.now().toString(), added_at: new Date().toISOString() };
              setContacts([...contacts, newContact]);
              addLog("ADD_CONTACT", "Owner (You)", `Added ${c.name}`);
            }}
            onRemove={(id: string) => {
              const contact = contacts.find(c => c.id === id);
              setContacts(contacts.filter(c => c.id !== id));
              if (contact) addLog("REMOVE_CONTACT", "Owner (You)", `Removed ${contact.name}`);
            }}
          />
        )}

        {activeTab === "guardians" && (
          <GuardianConfig 
            guardians={guardians}
            threshold={2}
            onUpdateGuardians={(addrs, threshold) => {
              // Real implementation would call API
              addLog("UPDATE_GUARDIANS", "Owner (You)", `Updated threshold to ${threshold}`);
            }}
          />
        )}

        {activeTab === "requests" && (
          <RequestInterface 
            requests={requests}
            isContact={true} // Mock: user can act as contact too
            isGuardian={true} // Mock: user can act as guardian too
            onSendRequest={() => {
              const newReq: EmergencyRequest = {
                id: `req_${Date.now()}`,
                requester_name: "You (Mock Contact)",
                requester_address: "GC2BK...",
                plan_id: "plan_001",
                status: "PENDING",
                created_at: new Date().toISOString(),
                approvals_count: 0,
                threshold_required: 2
              };
              setRequests([newReq, ...requests]);
              addLog("INITIATE_REQUEST", "You (Contact)", "Requested emergency access");
            }}
            onApprove={(id) => {
              setRequests(requests.map(r => r.id === id ? { ...r, approvals_count: r.approvals_count + 1, status: r.approvals_count + 1 >= r.threshold_required ? 'APPROVED' : 'PENDING' } : r));
              addLog("APPROVE_REQUEST", "You (Guardian)", `Approved request ${id}`);
            }}
            onReject={(id) => {
              setRequests(requests.map(r => r.id === id ? { ...r, status: 'REJECTED' } : r));
              addLog("REJECT_REQUEST", "You (Guardian)", `Rejected request ${id}`);
            }}
          />
        )}

        {activeTab === "logs" && (
          <AuditLog logs={logs} />
        )}
      </div>
    </div>
  );
}
