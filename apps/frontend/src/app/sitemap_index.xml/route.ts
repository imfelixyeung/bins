import { NextResponse } from "next/server";
import { BASE_URL, BUILD_TIME } from "../config";
import { PREMISES_SITEMAPS_URL } from "@/v2api/client";

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

const cachedSitemapIndex = async () => {
  const sitemaps: Sitemap[] = [
    {
      url: new URL("/sitemap.xml", BASE_URL).toString(),
      lastModified: BUILD_TIME,
    },
    {
      url: new URL("/docs/sitemap.xml", BASE_URL).toString(),
      lastModified: BUILD_TIME,
    },
    {
      url: new URL(PREMISES_SITEMAPS_URL, BASE_URL).toString(),
      lastModified: BUILD_TIME,
    },
  ];

  return await generateSitemapIndex(sitemaps);
};

export const GET = async () => {
  const xml = await cachedSitemapIndex();

  return new NextResponse(xml, {
    headers: {
      "Content-Type": "application/xml",
      "Content-Length": Buffer.byteLength(xml).toString(),
    },
  });
};

export const dynamic = "force-static";
