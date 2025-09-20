import { BASE_URL } from "@/app/config";
import { getPremisesSitemapPage, getPremisesSitemapPages } from "@/lib/sitemap";
import type { MetadataRoute } from "next";
import { notFound } from "next/navigation";

export const revalidate = 60;

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

  if (!pagePremises.length) notFound();

  return pagePremises.map(({ id, updatedAt }) => ({
    url: new URL(`/premises/${id}`, BASE_URL).toString(),
    lastModified: updatedAt,
  }));
};

export default sitemap;
