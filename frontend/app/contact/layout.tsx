import { generateSEO } from "@/lib/seo";

export const metadata = generateSEO({
  title: "Contact Support",
  description:
    "Get in touch with InheritX support team. We're here to help with any questions about inheritance planning and digital asset management.",
  url: "/contact",
  keywords: [
    "contact inheritx",
    "support",
    "help",
    "customer service",
    "inheritance support",
  ],
});

export default function ContactLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <>{children}</>;
}
