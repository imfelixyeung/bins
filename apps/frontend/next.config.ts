import nextra from "nextra";
import withSerwistInit from "@serwist/next";
import { NextConfig } from "next";

const revision = crypto.randomUUID();

const nextConfig: NextConfig = {
  output: "standalone",
  rewrites: async () => {
    if (process.env.NODE_ENV === "production") return [];
    return {
      beforeFiles: [
        {
          source: "/api/:path*",
          destination: "http://localhost:3000/api/:path*",
        },
      ],
    };
  },
};

const withNextra = nextra({
  defaultShowCopyCode: true,
  search: {
    codeblocks: false,
  },
  contentDirBasePath: "/docs",
});

const withSerwist = withSerwistInit({
  cacheOnNavigation: true,
  swSrc: "src/app/sw.ts",
  swDest: "public/sw.js",
  additionalPrecacheEntries: [{ url: "/~offline", revision }],
});

export default withSerwist(withNextra(nextConfig));
