import { Footer, Layout, Navbar } from "nextra-theme-docs";
import { Head } from "nextra/components";
import { getPageMap } from "nextra/page-map";
import "nextra-theme-docs/style.css";
import { ReactNode } from "react";
import CopyRight from "@/ui/copyright";

export const metadata = {
  title: {
    default: "Bins Docs",
    template: "%s | Bins Docs",
  },
};

const navbar = (
  <Navbar
    logo={<b>Bins Docs</b>}
    projectLink="https://github.com/imfelixyeung/bins"
  />
);

const footer = (
  <Footer>
    <CopyRight />
  </Footer>
);

// const sidebar = <Sidebar

export default async function RootLayout({
  children,
}: {
  children: ReactNode;
}) {
  return (
    <html lang="en" dir="ltr" suppressHydrationWarning>
      <Head />
      <body>
        <Layout
          navbar={navbar}
          pageMap={await getPageMap("/docs")}
          docsRepositoryBase="https://github.com/imfelixyeung/bins/tree/main/apps/web"
          footer={footer}
          feedback={{
            labels: "documentation",
          }}
          toc={{ backToTop: true }}
        >
          {children}
        </Layout>
      </body>
    </html>
  );
}
