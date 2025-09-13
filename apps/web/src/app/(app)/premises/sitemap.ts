import { BASE_URL } from "@/app/config";
import { getPremisesSitemapPages } from "@/lib/sitemap";
import type { MetadataRoute } from "next";

export const revalidate = 5;

const sitemap = async (): Promise<MetadataRoute.Sitemap> => {
  const pages = await getPremisesSitemapPages();

  return pages.map((page) => ({
    url: new URL(
      `/premises/chunks/sitemap/${page.id}.xml`,
      BASE_URL
    ).toString(),
    lastModified: new Date(),
  }));
};

export default sitemap;
