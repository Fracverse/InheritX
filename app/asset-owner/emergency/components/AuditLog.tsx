"use client";

import React from "react";
import { ListFilter, User, Shield, AlertTriangle } from "lucide-react";
import { AuditLog as LogType } from "@/app/lib/api/emergency";

interface Props {
  logs: LogType[];
}

export default function AuditLog({ logs }: Props) {
  const getIcon = (action: string) => {
    if (action.includes("ACTIVATE")) return <AlertTriangle size={14} className="text-red-500" />;
    if (action.includes("APPROVE")) return <Shield size={14} className="text-green-500" />;
    return <User size={14} className="text-[#33C5E0]" />;
  };

  return (
    <div className="bg-[#182024] rounded-2xl overflow-hidden border border-[#1C252A]">
      <div className="p-6 border-b border-[#1C252A] flex justify-between items-center">
        <h2 className="text-xl font-bold text-[#FCFFFF]">Audit Logs</h2>
        <button className="text-[#92A5A8] hover:text-[#FCFFFF] flex items-center gap-2">
          <ListFilter size={18} />
          Filter
        </button>
      </div>
      
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="text-[#92A5A8] text-xs uppercase tracking-wider border-b border-[#1C252A]">
              <th className="px-6 py-4 font-bold">Action</th>
              <th className="px-6 py-4 font-bold">Performed By</th>
              <th className="px-6 py-4 font-bold">Details</th>
              <th className="px-6 py-4 font-bold">Timestamp</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-[#1C252A]">
            {logs.map((log) => (
              <tr key={log.id} className="hover:bg-[#1C252A]/40 transition-colors">
                <td className="px-6 py-4">
                  <div className="flex items-center gap-2 text-sm font-medium text-[#FCFFFF]">
                    {getIcon(log.action)}
                    {log.action}
                  </div>
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center gap-2 text-sm text-[#92A5A8]">
                    <span className="truncate max-w-[120px] font-mono">{log.performed_by}</span>
                  </div>
                </td>
                <td className="px-6 py-4 text-sm text-[#92A5A8]">
                  {log.details || "-"}
                </td>
                <td className="px-6 py-4 text-sm text-[#92A5A8] whitespace-nowrap">
                  {new Date(log.timestamp).toLocaleString()}
                </td>
              </tr>
            ))}

            {logs.length === 0 && (
              <tr>
                <td colSpan={4} className="px-6 py-12 text-center text-[#92A5A8]">
                  No activities logged yet.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
