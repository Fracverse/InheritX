import { useState, useEffect } from "react";

interface Witness {
  id: string;
  name: string;
  email: string;
  wallet_address: string;
  status: "pending" | "signed";
}

interface Signature {
  witness_id: string;
  signed_at: string;
  wallet_address: string;
}

interface WitnessManagementProps {
  planId: string;
}

export default function WitnessManagement({ planId }: WitnessManagementProps) {
  const [witnesses, setWitnesses] = useState<Witness[]>([]);
  const [signatures, setSignatures] = useState<Signature[]>([]);
  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteName, setInviteName] = useState("");
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    fetchWitnesses();
    fetchSignatures();
  }, [planId]);

  const fetchWitnesses = async () => {
    try {
      const res = await fetch(`/api/witness/${planId}`);
      const data = await res.json();
      setWitnesses(data);
    } catch (err) {
      console.error("Failed to fetch witnesses", err);
    }
  };

  const fetchSignatures = async () => {
    try {
      const res = await fetch(`/api/witness/${planId}/signatures`);
      const data = await res.json();
      setSignatures(data);
    } catch (err) {
      console.error("Failed to fetch signatures", err);
    }
  };

  const inviteWitness = async () => {
    if (!inviteEmail || !inviteName) return;
    setLoading(true);
    try {
      const res = await fetch("/api/witness/invite", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ plan_id: planId, email: inviteEmail, name: inviteName }),
      });
      if (res.ok) {
        setMessage("Witness invited successfully!");
        setInviteEmail("");
        setInviteName("");
        fetchWitnesses();
      }
    } catch (err) {
      setMessage("Failed to invite witness.");
    } finally {
      setLoading(false);
    }
  };

  const signWill = async (witnessId: string) => {
    setLoading(true);
    try {
      const res = await fetch("/api/witness/sign", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ plan_id: planId, witness_id: witnessId }),
      });
      if (res.ok) {
        setMessage("Will signed successfully!");
        fetchWitnesses();
        fetchSignatures();
      }
    } catch (err) {
      setMessage("Failed to sign will.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6 max-w-2xl mx-auto space-y-8">
      <h2 className="text-2xl font-bold text-gray-800">Witness Management</h2>

      {/* Invite Witness Form */}
      <div className="bg-white rounded-xl shadow p-6 space-y-4">
        <h3 className="text-lg font-semibold text-gray-700">Invite a Witness</h3>
        <input
          type="text"
          placeholder="Witness Name"
          value={inviteName}
          onChange={(e) => setInviteName(e.target.value)}
          className="w-full border rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <input
          type="email"
          placeholder="Witness Email"
          value={inviteEmail}
          onChange={(e) => setInviteEmail(e.target.value)}
          className="w-full border rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          onClick={inviteWitness}
          disabled={loading}
          className="bg-blue-600 text-white px-6 py-2 rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
        >
          {loading ? "Sending..." : "Send Invitation"}
        </button>
        {message && <p className="text-sm text-green-600">{message}</p>}
      </div>

      {/* Witness List */}
      <div className="bg-white rounded-xl shadow p-6 space-y-4">
        <h3 className="text-lg font-semibold text-gray-700">Witnesses</h3>
        {witnesses.length === 0 ? (
          <p className="text-sm text-gray-400">No witnesses added yet.</p>
        ) : (
          witnesses.map((w) => (
            <div key={w.id} className="flex items-center justify-between border-b pb-3">
              <div>
                <p className="font-medium text-gray-800">{w.name}</p>
                <p className="text-sm text-gray-500">{w.email}</p>
                <span className={`text-xs font-semibold ${w.status === "signed" ? "text-green-600" : "text-yellow-500"}`}>
                  {w.status === "signed" ? "✓ Signed" : "⏳ Pending"}
                </span>
              </div>
              {w.status !== "signed" && (
                <button
                  onClick={() => signWill(w.id)}
                  disabled={loading}
                  className="bg-green-600 text-white px-4 py-1.5 rounded-lg text-sm hover:bg-green-700 disabled:opacity-50"
                >
                  Sign Will
                </button>
              )}
            </div>
          ))
        )}
      </div>

      {/* Signatures List */}
      <div className="bg-white rounded-xl shadow p-6 space-y-4">
        <h3 className="text-lg font-semibold text-gray-700">Signatures</h3>
        {signatures.length === 0 ? (
          <p className="text-sm text-gray-400">No signatures yet.</p>
        ) : (
          signatures.map((sig, i) => (
            <div key={i} className="border-b pb-3">
              <p className="text-sm text-gray-700 font-medium">{sig.wallet_address}</p>
              <p className="text-xs text-gray-400">Signed at: {new Date(sig.signed_at).toLocaleString()}</p>
            </div>
          ))
        )}
      </div>
    </div>
  );
}