"use client";

import { useReportWebVitals } from "next/web-vitals";

export function Vitals() {
  useReportWebVitals((metric) => {
    if (process.env.NODE_ENV === "development") {
      console.log(metric);
    }
    // You could send this to an analytics endpoint here
  });

  return null;
}
