import nextra from "nextra";
import withSerwistInit from "@serwist/next";

const revision = crypto.randomUUID();

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
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
