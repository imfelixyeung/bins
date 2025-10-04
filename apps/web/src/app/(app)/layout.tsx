import { cn } from "@/lib/utils";
import "@/app/globals.css";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import Providers from "../providers";
import Link from "next/link";
import CopyRight from "@/ui/copyright";
import { Metadata } from "next";
import ThemeSwitch from "@/ui/theme-switch";

export const metadata: Metadata = {
  title: { default: "Check your bin days - Bins", template: "%s - Bins" },
  description: "Find out when each of your bins are scheduled to be emptied",
  metadataBase: new URL("https://bins.felixyeung.com"),
  openGraph: {
    title: "Check your bin days",
    description: "Find out when each of your bins are scheduled to be emptied",
    images: [
      {
        url: "/api/og",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Check your bin days",
    description: "Find out when each of your bins are scheduled to be emptied",
    images: [
      {
        url: "/api/og",
      },
    ],
  },
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={cn(GeistSans.variable, GeistMono.variable)}
      suppressHydrationWarning
    >
      <body>
        <Providers>
          <div className="min-h-dvh flex flex-col">
            <header className="border-b h-14 flex flex-col justify-center">
              <nav className="container">
                <Link href="/" className="text-xl font-bold">
                  Bins
                </Link>
              </nav>
            </header>
            <main className="grow flex flex-col">{children}</main>
          </div>
          <footer className="border-t py-6 text-muted-foreground text-sm">
            <div className="container grid md:grid-cols-2 gap-8">
              <div className="max-w-prose">
                <p>
                  Data is provided by{" "}
                  <Link
                    href="https://www.leeds.gov.uk/opendata"
                    rel="noreferrer"
                    className="underline"
                  >
                    Leeds City Council
                  </Link>{" "}
                  in the{" "}
                  <Link
                    href="https://datamillnorth.org/dataset/ep6lz/household-waste-collections"
                    rel="noreferrer"
                    className="underline"
                  >
                    Household Waste Collection dataset
                  </Link>
                  , available under the{" "}
                  <Link
                    href="https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/"
                    rel="noreferrer"
                    className="underline"
                  >
                    Open Government Licence
                  </Link>
                  .
                </p>
                <p className="mt-2">
                  Contact{" "}
                  <Link
                    href="mailto:me@felixyeung.com"
                    rel="noreferrer"
                    className="underline"
                  >
                    me@felixyeung.com
                  </Link>
                  .
                </p>
              </div>
              <div className="flex flex-col md:items-end md:text-end">
                <p className="font-bold text-lg">For Developers</p>
                <ul>
                  <li>
                    <Link
                      href="/docs"
                      rel="noreferrer"
                      className="hover:underline"
                    >
                      About
                    </Link>
                  </li>
                  <li>
                    <Link
                      href="/docs/api"
                      rel="noreferrer"
                      className="hover:underline"
                    >
                      API Reference
                    </Link>
                  </li>
                </ul>
              </div>
            </div>
            <div className="container text-muted-foreground mt-6 flex items-center justify-between gap-3 flex-wrap-reverse">
              <p>
                <CopyRight />
              </p>
              <ThemeSwitch />
            </div>
          </footer>
        </Providers>
      </body>
    </html>
  );
}
