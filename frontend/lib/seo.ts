import { Metadata } from "next";

export interface SEOProps {
  title?: string;
  description?: string;
  keywords?: string[];
  image?: string;
  url?: string;
  type?: "website" | "article";
  publishedTime?: string;
  modifiedTime?: string;
  author?: string;
  noindex?: boolean;
}

const defaultSEO = {
  siteName: "InheritX",
  domain: "https://inheritx.com",
  defaultTitle: "InheritX - Secure Wealth Inheritance & Asset Planning",
  defaultDescription:
    "InheritX helps you plan and share your assets securely with loved ones. Simple inheritance planning, custom beneficiary rules, and stress-free transfers.",
  defaultImage: "/og-image.jpg",
  twitterHandle: "@inheritx",
};

export function generateSEO({
  title,
  description,
  keywords,
  image,
  url,
  type = "website",
  publishedTime,
  modifiedTime,
  author,
  noindex = false,
}: SEOProps = {}): Metadata {
  const pageTitle = title
    ? `${title} | ${defaultSEO.siteName}`
    : defaultSEO.defaultTitle;
  const pageDescription = description || defaultSEO.defaultDescription;
  const pageImage = image
    ? `${defaultSEO.domain}${image}`
    : `${defaultSEO.domain}${defaultSEO.defaultImage}`;
  const pageUrl = url ? `${defaultSEO.domain}${url}` : defaultSEO.domain;

  const metadata: Metadata = {
    title: pageTitle,
    description: pageDescription,
    keywords: keywords || [
      "wealth inheritance",
      "asset planning",
      "digital inheritance",
      "estate planning",
      "legacy planning",
      "beneficiary management",
      "secure transfers",
      "blockchain inheritance",
      "crypto inheritance",
      "digital assets",
    ],
    authors: author ? [{ name: author }] : [{ name: "InheritX Team" }],
    creator: "InheritX",
    publisher: "InheritX",
    robots: noindex
      ? "noindex, nofollow"
      : "index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1",
    openGraph: {
      type,
      locale: "en_US",
      url: pageUrl,
      siteName: defaultSEO.siteName,
      title: pageTitle,
      description: pageDescription,
      images: [
        {
          url: pageImage,
          width: 1200,
          height: 630,
          alt: title || defaultSEO.defaultTitle,
        },
      ],
      ...(publishedTime && { publishedTime }),
      ...(modifiedTime && { modifiedTime }),
    },
    twitter: {
      card: "summary_large_image",
      site: defaultSEO.twitterHandle,
      creator: defaultSEO.twitterHandle,
      title: pageTitle,
      description: pageDescription,
      images: [pageImage],
    },
    alternates: {
      canonical: pageUrl,
    },
  };

  return metadata;
}

export const defaultMetadata = generateSEO();
