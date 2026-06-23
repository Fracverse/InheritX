"use client";

import React, { use } from "react";
import { AdminAuthProvider } from "@/context/AdminAuthContext";
import PlanDetailContent from "./PlanDetailContent";

interface PageProps {
  params: Promise<{ id: string }>;
}

export default function AdminPlanDetailPage({ params }: PageProps) {
  const resolvedParams = use(params);
  return (
    <AdminAuthProvider>
      <PlanDetailContent id={resolvedParams.id} />
    </AdminAuthProvider>
  );
}
