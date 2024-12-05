import CopyRight from "@/ui/copyright";
import { useConfig, type DocsThemeConfig } from "nextra-theme-docs";

export default {
  logo: <b>Bins Docs</b>,
  project: {
    link: "https://github.com/imfelixyeung/bins",
    icon: null, // hide github
  },
  docsRepositoryBase: "https://github.com/imfelixyeung/bins/tree/main/apps/web",
  head: () => {
    const frontMatter = useConfig();
    const { title } = frontMatter;
    const fullTitle = `${title} - Bins Docs`;
    return (
      <>
        <title>{fullTitle}</title>
        <meta property="og:title" content={fullTitle} />
      </>
    );
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
    content: <CopyRight />,
  },
} satisfies DocsThemeConfig;
