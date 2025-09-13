import { getPremisesSitemapPages } from "@/lib/sitemap";
import { NextResponse } from "next/server";
import { BASE_URL, BUILD_TIME } from "../config";

export const dynamic = "force-static";
export const revalidate = 60;

type Sitemap = { url: string; lastModified?: Date };

const generateSitemapIndex = async (sitemaps: Sitemap[]) => {
  const sitemapStrings = sitemaps.map((sitemap) => {
    return `
  <sitemap>
    <loc>${sitemap.url}</loc>
    ${sitemap.lastModified ? `<lastmod>${sitemap.lastModified.toISOString()}</lastmod>` : ""}
  </sitemap>
    `;
  });

  return `<?xml version="1.0" encoding="UTF-8"?>
  <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  ${sitemapStrings.join("\n")}
</sitemapindex>
  `;
};

export const GET = async () => {
  const pages = await getPremisesSitemapPages();

  const sitemaps: Sitemap[] = [
    {
      url: new URL("/sitemap.xml", BASE_URL).toString(),
      lastModified: BUILD_TIME,
    },
    {
      url: new URL("/docs/sitemap.xml", BASE_URL).toString(),
      lastModified: BUILD_TIME,
    },
    ...pages.map((page) => ({
      url: new URL(`/premises/sitemap/${page.id}.xml`, BASE_URL).toString(),
    })),
  ];

  const xml = await generateSitemapIndex(sitemaps);

  return new NextResponse(xml, {
    headers: {
      "Content-Type": "application/xml",
      "Content-Length": Buffer.byteLength(xml).toString(),
    },
  });
};
