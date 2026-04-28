"use client";

import React from "react";
import { Send, CheckCircle2, XCircle, Users, Activity } from "lucide-react";
import { EmergencyRequest } from "@/app/lib/api/emergency";

interface Props {
  requests: EmergencyRequest[];
  isContact: boolean;
  isGuardian: boolean;
  onSendRequest: () => void;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
}

export default function RequestInterface({ 
  requests, 
  isContact, 
  isGuardian, 
  onSendRequest, 
  onApprove, 
  onReject 
}: Props) {
  return (
    <div className="space-y-8">
      {isContact && (
        <div className="bg-gradient-to-r from-[#33C5E0]/10 to-transparent p-6 rounded-2xl border border-[#33C5E0]/20">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 bg-[#33C5E0] text-[#161E22] rounded-full flex items-center justify-center">
                <Send size={24} />
              </div>
              <div>
                <h3 className="text-lg font-bold text-[#FCFFFF]">Request Emergency Access</h3>
                <p className="text-sm text-[#92A5A8]">Initiate the emergency protocol if you believe the owner is unreachable.</p>
              </div>
            </div>
            <button
              onClick={onSendRequest}
              className="px-8 py-3 bg-[#33C5E0] text-[#161E22] rounded-xl font-bold hover:bg-[#2AB8D3] transition-all hover:scale-105 active:scale-95"
            >
              INITIATE REQUEST
            </button>
          </div>
        </div>
      )}

      <div className="space-y-4">
        <h3 className="text-lg font-bold text-[#FCFFFF] flex items-center gap-2">
          <Activity size={20} className="text-[#33C5E0]" />
          Active Requests
        </h3>
        
        <div className="space-y-3">
          {requests.map((request) => (
            <div key={request.id} className="bg-[#182024] p-6 rounded-2xl border border-[#1C252A]">
              <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 bg-[#1C252A] rounded-full flex items-center justify-center text-[#33C5E0]">
                    <Users size={20} />
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-[#FCFFFF]">{request.requester_name}</span>
                      <span className="px-2 py-0.5 bg-[#1C252A] text-[#92A5A8] text-[10px] rounded uppercase">Requester</span>
                    </div>
                    <p className="text-xs text-[#92A5A8] mt-1">Requested on {new Date(request.created_at).toLocaleString()}</p>
                  </div>
                </div>

                <div className="flex flex-1 items-center gap-4">
                  <div className="flex-1">
                    <div className="flex justify-between text-xs mb-1.5">
                      <span className="text-[#92A5A8]">Guardian Approvals</span>
                      <span className="text-[#33C5E0] font-bold">{request.approvals_count} / {request.threshold_required}</span>
                    </div>
                    <div className="w-full h-1.5 bg-[#1C252A] rounded-full overflow-hidden">
                      <div 
                        className="bg-[#33C5E0] h-full transition-all duration-500" 
                        style={{ width: `${(request.approvals_count / request.threshold_required) * 100}%` }} 
                      />
                    </div>
                  </div>

                  {isGuardian && request.status === 'PENDING' && (
                    <div className="flex gap-2">
                      <button
                        onClick={() => onReject(request.id)}
                        className="p-3 text-red-500 hover:bg-red-500/10 rounded-xl transition-colors"
                        title="Reject Request"
                      >
                        <XCircle size={24} />
                      </button>
                      <button
                        onClick={() => onApprove(request.id)}
                        className="p-3 text-green-500 hover:bg-green-500/10 rounded-xl transition-colors"
                        title="Approve Request"
                      >
                        <CheckCircle2 size={24} />
                      </button>
                    </div>
                  )}
                </div>

                <div className="flex items-center gap-2">
                  <span className={`px-3 py-1 rounded-full text-xs font-bold ${
                    request.status === 'APPROVED' ? 'bg-green-500/10 text-green-500' :
                    request.status === 'REJECTED' ? 'bg-red-500/10 text-red-500' :
                    'bg-yellow-500/10 text-yellow-500'
                  }`}>
                    {request.status}
                  </span>
                </div>
              </div>
            </div>
          ))}

          {requests.length === 0 && (
            <div className="py-12 text-center bg-[#182024]/30 rounded-2xl border border-[#1C252A]">
              <p className="text-[#92A5A8]">No active emergency requests found.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
