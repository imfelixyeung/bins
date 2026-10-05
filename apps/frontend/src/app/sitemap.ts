import { BASE_URL, BUILD_TIME } from "@/app/config";
import type { MetadataRoute } from "next";

const sitemap = (): MetadataRoute.Sitemap => {
  return [
    {
      url: new URL(BASE_URL).toString(),
      lastModified: BUILD_TIME,
      changeFrequency: "yearly",
      priority: 1,
    },
  ];
};

export default sitemap;
