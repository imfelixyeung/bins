import { BASE_URL } from "@/app/config";
import { getPremisesSitemapPage, getPremisesSitemapPages } from "@/lib/sitemap";
import type { MetadataRoute } from "next";

export const revalidate = 5;

export const generateSitemaps = async () => {
  const pages = await getPremisesSitemapPages();
  return pages;
};

const sitemap = async ({
  id,
}: {
  id: number;
}): Promise<MetadataRoute.Sitemap> => {
  const pagePremises = await getPremisesSitemapPage(id);

  return pagePremises.map(({ id, updatedAt }) => ({
    url: new URL(`/premises/${id}`, BASE_URL).toString(),
    lastModified: updatedAt,
  }));
};

export default sitemap;
