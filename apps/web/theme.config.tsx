import CopyRight from "@/ui/copyright";
import { useConfig, type DocsThemeConfig } from "nextra-theme-docs";

export default {
  logo: <b>Bins Docs</b>,
  project: {
    link: "https://github.com/imfelixyeung/bins",
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
  editLink: {},
  feedback: {
    labels: "documentation",
  },
  footer: {
    content: <CopyRight />,
  },
} satisfies DocsThemeConfig;
