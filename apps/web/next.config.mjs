import nextra from "nextra";

/** @type {import('next').NextConfig} */
const nextConfig = {};

const withNextra = nextra({
  theme: "nextra-theme-docs",
  themeConfig: "./theme.config.tsx",
  defaultShowCopyCode: true,
  search: {
    codeblocks: false,
  },
});

export default withNextra(nextConfig);
