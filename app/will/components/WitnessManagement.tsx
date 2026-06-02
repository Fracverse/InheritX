"use client";

import { useState, useEffect } from "react";

interface Witness {
  id: string;
  name: string;
  email: string;
  status: "pending" | "signed";
  invited_at?: string | null;
  signed_at?: string | null;
  verified_on_chain?: boolean;
  verification_error?: string | null;
}

interface WitnessManagementProps {
  planId: string;
}

export default function WitnessManagement({ planId }: WitnessManagementProps) {
  const [witnesses, setWitnesses] = useState<Witness[]>([]);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [notifications, setNotifications] = useState<string[]>([]);

  const inviteWitness = async () => {
    if (!name || !email) {
      setError("Name and email are required.");
      return;
    }
    setLoading(true);
    setError("");
    setSuccess("");
    try {
      const res = await fetch("/api/witness/invite", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name, email, plan_id: planId }),
      });
      if (!res.ok) throw new Error("Failed");
      const data = await res.json();
      // API should return the created witness object
      // Refresh list to ensure canonical data
      await fetchWitnesses();
      setSuccess(name + " invited successfully!");
      setNotifications((n) => [...n, `${name} invited`]);
      setName("");
      setEmail("");
    } catch {
      setError("Failed to invite witness.");
    } finally {
      setLoading(false);
    }
  };

  const addNotification = (msg: string) => {
    setNotifications((n) => [msg, ...n].slice(0, 5));
    setTimeout(() => setNotifications((n) => n.slice(0, n.length - 1)), 5000);
  };

  const fetchWitnesses = async () => {
    if (!planId) return;
    try {
      const res = await fetch(`/api/witness/${planId}`);
      if (!res.ok) return;
      const data: Witness[] = await res.json();
      setWitnesses(data || []);
    } catch (e) {
      // ignore
    }
  };

  const signWitness = async (witnessId: string) => {
    setLoading(true);
    setError("");
    try {
      const res = await fetch(`/api/witness/sign`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ witness_id: witnessId, plan_id: planId }),
      });
      if (!res.ok) throw new Error("sign failed");
      await fetchWitnesses();
      addNotification("Witness signed");
    } catch (e) {
      setError("Failed to sign witness.");
    } finally {
      setLoading(false);
    }
  };

  const verifySignature = async (witnessId: string) => {
    try {
      const res = await fetch(`/api/witness/verify`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ witness_id: witnessId, plan_id: planId }),
      });
      if (!res.ok) throw new Error("verify failed");
      const result = await res.json();
      if (result.verified) {
        setWitnesses((prev) =>
          prev.map((w) =>
            w.id === witnessId
              ? { ...w, verified_on_chain: true, verification_error: null }
              : w
          )
        );
        addNotification("Signature verified on-chain");
      } else {
        setWitnesses((prev) =>
          prev.map((w) =>
            w.id === witnessId
              ? { ...w, verified_on_chain: false, verification_error: result.error || "Not found on-chain" }
              : w
          )
        );
      }
    } catch (e) {
      addNotification("Verification check failed");
    }
  };

  useEffect(() => {
    if (!planId) return;
    let mounted = true;
    const fetchWitnesses = async () => {
      try {
        const res = await fetch(`/api/witness/${planId}`);
        if (!res.ok) return;
        const data: Witness[] = await res.json();
        if (mounted) setWitnesses(data || []);
      } catch (e) {
        // ignore for now
      }
    };
    fetchWitnesses();
    return () => {
      mounted = false;
    };
  }, [planId]);

  useEffect(() => {
    if (!witnesses.length) return;
    witnesses.forEach((w) => {
      if (w.status === "signed" && !w.verified_on_chain && !w.verification_error) {
        verifySignature(w.id);
      }
    });
  }, [witnesses]);


  return (
    <div className="p-6 bg-white rounded-lg shadow">
      {/* Notifications */}
      {notifications.length > 0 && (
        <div className="mb-4">
          {notifications.map((n, i) => (
            <div key={i} className="bg-blue-50 border-l-4 border-blue-400 p-2 mb-1 text-sm">
              {n}
            </div>
          ))}
        </div>
      )}
      <h2 className="text-2xl font-bold mb-6">Witness Management</h2>
      <div className="mb-8 p-4 border rounded-lg">
        <h3 className="text-lg font-semibold mb-4">Invite a Witness</h3>
        {error && <p className="text-red-500 mb-3">{error}</p>}
        {success && <p className="text-green-500 mb-3">{success}</p>}
        <input
          className="w-full border rounded p-2 mb-3"
          placeholder="Witness Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <input
          className="w-full border rounded p-2 mb-3"
          placeholder="Witness Email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
        <button
          className="bg-blue-600 text-white px-4 py-2 rounded w-full"
          onClick={inviteWitness}
          disabled={loading}
        >
          {loading ? "Sending..." : "Send Invitation"}
        </button>
      </div>
      <div>
        <h3 className="text-lg font-semibold mb-4">Witnesses ({witnesses.length})</h3>
        {witnesses.length === 0 ? (
          <p className="text-gray-400">No witnesses invited yet.</p>
        ) : (
          witnesses.map((w) => (
            <div key={w.id} className="border-b py-3 flex justify-between items-start">
              <div>
                <p className="font-medium">{w.name}</p>
                <p className="text-sm text-gray-500">{w.email}</p>
                {w.invited_at && (
                  <p className="text-xs text-gray-400">Invited: {new Date(w.invited_at).toLocaleString()}</p>
                )}
                {w.signed_at && (
                  <p className="text-xs text-gray-400">Signed: {new Date(w.signed_at).toLocaleString()}</p>
                )}
                {w.verified_on_chain && (
                  <p className="text-xs text-green-500 font-semibold">✓ Verified on-chain</p>
                )}
                {w.verification_error && (
                  <p className="text-xs text-red-500">Verification: {w.verification_error}</p>
                )}
              </div>
              <div className="flex flex-col items-end gap-2">
                <span className={w.status === "signed" ? "text-green-500 font-semibold" : "text-yellow-500 font-semibold"}>
                  {w.status === "signed" ? "Signed" : "Pending"}
                </span>
                {w.status !== "signed" && (
                  <button
                    className="text-sm text-blue-600 underline"
                    onClick={() => signWitness(w.id)}
                    disabled={loading}
                  >
                    Sign
                  </button>
                )}
                {w.status === "signed" && !w.verified_on_chain && (
                  <button
                    className="text-sm text-purple-600 underline"
                    onClick={() => verifySignature(w.id)}
                    disabled={loading}
                  >
                    Verify
                  </button>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}