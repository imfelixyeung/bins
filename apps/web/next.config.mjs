import nextra from "nextra";

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

export default withNextra(nextConfig);
