"use client";

import { useState, useEffect } from "react";

interface Proposal {
  id: string;
  title: string;
  description: string;
  status: "active" | "passed" | "failed" | "executed";
  votes_for: number;
  votes_against: number;
  voting_end: string;
  proposer: string;
}

export default function GovernancePage() {
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [view, setView] = useState<"list" | "create" | "detail">("list");
  const [selected, setSelected] = useState<Proposal | null>(null);
  const [form, setForm] = useState({ title: "", description: "", type: "general", voting_period: "7" });
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => { fetchProposals(); }, []);

  const fetchProposals = async () => {
    try {
      const res = await fetch("/api/governance/proposals");
      const data = await res.json();
      setProposals(data);
    } catch (err) { console.error(err); }
  };

  const createProposal = async () => {
    setLoading(true);
    try {
      const res = await fetch("/api/governance/proposals", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form),
      });
      if (res.ok) {
        setMessage("Proposal created!");
        setView("list");
        fetchProposals();
      }
    } catch (err) { setMessage("Failed to create proposal."); }
    finally { setLoading(false); }
  };

  const vote = async (id: string, voteType: "for" | "against") => {
    setLoading(true);
    try {
      await fetch(`/api/governance/proposals/${id}/vote`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ vote: voteType }),
      });
      setMessage("Vote submitted!");
      fetchProposals();
    } catch (err) { setMessage("Vote failed."); }
    finally { setLoading(false); }
  };

  const execute = async (id: string) => {
    setLoading(true);
    try {
      await fetch(`/api/governance/proposals/${id}/execute`, { method: "POST" });
      setMessage("Proposal executed!");
      fetchProposals();
    } catch (err) { setMessage("Execution failed."); }
    finally { setLoading(false); }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold text-gray-800">Governance</h1>
        <button
          onClick={() => setView(view === "create" ? "list" : "create")}
          className="bg-blue-600 text-white px-5 py-2 rounded-lg text-sm font-medium hover:bg-blue-700"
        >
          {view === "create" ? "← Back" : "+ New Proposal"}
        </button>
      </div>

      {message && <p className="text-sm text-green-600 bg-green-50 p-3 rounded-lg">{message}</p>}

      {/* Create Proposal Form */}
      {view === "create" && (
        <div className="bg-white rounded-xl shadow p-6 space-y-4">
          <h2 className="text-xl font-semibold text-gray-700">Create Proposal</h2>
          <input
            placeholder="Title"
            value={form.title}
            onChange={e => setForm({ ...form, title: e.target.value })}
            className="w-full border rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <textarea
            placeholder="Description"
            value={form.description}
            onChange={e => setForm({ ...form, description: e.target.value })}
            rows={4}
            className="w-full border rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <select
            value={form.type}
            onChange={e => setForm({ ...form, type: e.target.value })}
            className="w-full border rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="general">General</option>
            <option value="parameter">Parameter Update</option>
            <option value="execution">Execution</option>
          </select>
          <input
            placeholder="Voting period (days)"
            value={form.voting_period}
            onChange={e => setForm({ ...form, voting_period: e.target.value })}
            className="w-full border rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={createProposal}
            disabled={loading}
            className="bg-blue-600 text-white px-6 py-2 rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? "Submitting..." : "Submit Proposal"}
          </button>
        </div>
      )}

      {/* Proposal List */}
      {view === "list" && (
        <div className="space-y-4">
          {proposals.length === 0 ? (
            <p className="text-gray-400 text-sm">No proposals yet.</p>
          ) : (
            proposals.map(p => (
              <div key={p.id} className="bg-white rounded-xl shadow p-5 space-y-3">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold text-gray-800">{p.title}</h3>
                  <span className={`text-xs font-bold px-2 py-1 rounded-full ${
                    p.status === "active" ? "bg-blue-100 text-blue-700" :
                    p.status === "passed" ? "bg-green-100 text-green-700" :
                    p.status === "executed" ? "bg-purple-100 text-purple-700" :
                    "bg-red-100 text-red-700"
                  }`}>{p.status.toUpperCase()}</span>
                </div>
                <p className="text-sm text-gray-500">{p.description}</p>
                <div className="flex gap-2 text-sm">
                  <span className="text-green-600">✓ {p.votes_for} For</span>
                  <span className="text-red-500">✗ {p.votes_against} Against</span>
                </div>
                <div className="flex gap-2">
                  {p.status === "active" && (
                    <>
                      <button onClick={() => vote(p.id, "for")} className="bg-green-600 text-white px-4 py-1.5 rounded-lg text-sm hover:bg-green-700">Vote For</button>
                      <button onClick={() => vote(p.id, "against")} className="bg-red-500 text-white px-4 py-1.5 rounded-lg text-sm hover:bg-red-600">Vote Against</button>
                    </>
                  )}
                  {p.status === "passed" && (
                    <button onClick={() => execute(p.id)} className="bg-purple-600 text-white px-4 py-1.5 rounded-lg text-sm hover:bg-purple-700">Execute</button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}