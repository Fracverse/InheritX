import { generateSEO } from "@/lib/seo";

export const metadata = generateSEO({
  title: "Claims",
  description:
    "Submit and manage inheritance claims. Track claim status and access your inherited digital assets.",
  url: "/asset-owner/claim",
  keywords: [
    "inheritance claims",
    "asset claims",
    "beneficiary claims",
    "claim management",
    "inherited assets",
  ],
  noindex: true,
});

export default function ClaimLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <>{children}</>;
}
