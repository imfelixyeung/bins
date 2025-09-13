import { BASE_URL } from "@/app/config";
import type { MetadataRoute } from "next";

const sitemap = (): MetadataRoute.Sitemap => {
  return [
    {
      url: new URL(BASE_URL).toString(),
      lastModified: new Date(),
      changeFrequency: "yearly",
      priority: 1,
    },
    {
      url: new URL("/docs/sitemap.xml", BASE_URL).toString(),
      lastModified: new Date(),
      changeFrequency: "monthly",
      priority: 0.8,
    },
    {
      url: new URL("/premises/sitemap.xml", BASE_URL).toString(),
      lastModified: new Date(),
      changeFrequency: "weekly",
      priority: 0.5,
    },
  ];
};

export default sitemap;
