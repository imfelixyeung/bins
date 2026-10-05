import { BASE_URL, BUILD_TIME } from "@/app/config";
import type { MetadataRoute } from "next";

const sitemap = (): MetadataRoute.Sitemap => {
  return [
    {
      url: new URL("/docs", BASE_URL).toString(),
      lastModified: BUILD_TIME,
      changeFrequency: "monthly",
      priority: 1,
    },
    {
      url: new URL("/docs/api", BASE_URL).toString(),
      lastModified: BUILD_TIME,
      changeFrequency: "monthly",
      priority: 0.8,
    },
    {
      url: new URL("/docs/calendar", BASE_URL).toString(),
      lastModified: BUILD_TIME,
      changeFrequency: "monthly",
      priority: 0.8,
    },
  ];
};

export default sitemap;
