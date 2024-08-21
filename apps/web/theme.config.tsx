import CopyRight from "@/ui/copyright";
import type { DocsThemeConfig } from "nextra-theme-docs";

export default {
  logo: <b>Bins Docs</b>,
  project: {
    link: "https://github.com/felixyeungdev/bins",
    icon: null, // hide github
  },
  docsRepositoryBase:
    "https://github.com/felixyeungdev/bins/tree/main/apps/web",
  useNextSeoProps() {
    return {
      titleTemplate: "%s - Bins Docs",
    };
  },
  toc: {
    backToTop: true,
  },
  editLink: {
    component: null, // hide github
  },
  feedback: {
    content: null, // hide github
  },
  footer: {
    text: <CopyRight />,
  },
} satisfies DocsThemeConfig;
