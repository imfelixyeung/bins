import { join } from "node:path";
import withSerwistInit from "@serwist/next";
import type { NextConfig } from "next";
import nextra from "nextra";

const revision = crypto.randomUUID();

const DEVELOPMENT_BACKEND_URL =
  process.env.NODE_ENV === "development"
    ? process.env.DEVELOPMENT_BACKEND_URL
    : null;

const nextConfig: NextConfig = {
  // Hack to enable rewrites, export doesn't do rewrites in development mode.
  output: DEVELOPMENT_BACKEND_URL ? "standalone" : "export",
  turbopack: {
    root: join(__dirname, "../.."),
  },
  rewrites: async () => {
    if (!DEVELOPMENT_BACKEND_URL) return [];
    return {
      beforeFiles: [
        {
          source: "/api/:path*",
          destination: new URL(
            "/api/:path*",
            DEVELOPMENT_BACKEND_URL,
          ).toString(),
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
  disable: process.env.NODE_ENV !== "production",
});

export default withSerwist(withNextra(nextConfig));
