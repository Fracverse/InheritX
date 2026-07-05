"use client";

import { useState, useEffect } from "react";
import { useRouter, useParams } from "next/navigation";
import { plansAPI } from "@/app/lib/api/plans";
import type { Plan } from "@/app/lib/api/plans";
import { EditInheritancePlanPanel } from "@/components/plans/EditInheritancePlanPanel";
import { Skeleton } from "@/components/ui/Skeleton";
import { AlertCircle } from "lucide-react";

export default function EditPlanPage() {
  const router = useRouter();
  const params = useParams();
  const planId = params.planId as string;

  const [plan, setPlan] = useState<Plan | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!planId) return;

    plansAPI
      .getPlan(planId)
      .then(setPlan)
      .catch((err) =>
        setError(err instanceof Error ? err.message : "Failed to load plan.")
      )
      .finally(() => setLoading(false));
  }, [planId]);

  const handleClose = () => router.back();

  const handleSaved = (updated: Plan) => {
    setPlan(updated);
    router.back();
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-[#161E22] flex items-center justify-center p-6">
        <div className="w-full max-w-2xl space-y-4">
          <Skeleton className="h-8 w-48" />
          <Skeleton className="h-64 w-full rounded-2xl" />
        </div>
      </div>
    );
  }

  if (error || !plan) {
    return (
      <div className="min-h-screen bg-[#161E22] flex items-center justify-center p-6">
        <div className="flex flex-col items-center gap-3 text-[#F56565]">
          <AlertCircle size={32} />
          <p className="text-sm">{error ?? "Plan not found."}</p>
          <button
            onClick={handleClose}
            className="mt-2 text-xs text-[#92A5A8] underline hover:text-white"
          >
            Go back
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#161E22]">
      <EditInheritancePlanPanel
        plan={plan}
        onClose={handleClose}
        onSaved={handleSaved}
      />
    </div>
  );
}
