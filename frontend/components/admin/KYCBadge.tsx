import { KYCStatus } from "@/lib/adminTypes";

const styles: Record<KYCStatus, string> = {
  approved: "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20",
  pending: "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20",
  rejected: "bg-red-500/10 text-red-400 border border-red-500/20",
};

const labels: Record<KYCStatus, string> = {
  approved: "Approved",
  pending: "Pending",
  rejected: "Rejected",
};

export function KYCBadge({ status }: { status: KYCStatus }) {
  return (
    <span
      className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium ${styles[status]}`}
    >
      <span
        className={`w-1.5 h-1.5 rounded-full mr-1.5 ${
          status === "approved"
            ? "bg-emerald-400"
            : status === "pending"
            ? "bg-yellow-400"
            : "bg-red-400"
        }`}
      />
      {labels[status]}
    </span>
  );
}
