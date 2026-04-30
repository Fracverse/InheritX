import { generateSEO } from "@/lib/seo";

export const metadata = generateSEO({
  title: "About Us",
  description:
    "Learn about InheritX - a blockchain-powered platform dedicated to redefining how digital assets are secured and passed down through generations.",
  url: "/about",
  keywords: [
    "about inheritx",
    "blockchain inheritance",
    "digital estate planning",
    "crypto inheritance platform",
    "secure asset transfer",
  ],
});

export default function AboutLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <>{children}</>;
}
