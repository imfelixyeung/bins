import { cn } from "@/lib/utils";
import "./globals.css";
import { GeistSans } from "geist/font/sans";
import Providers from "./providers";
import Link from "next/link";
import CopyRight from "@/ui/copyright";
import { Metadata } from "next";

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
    <html lang="en" className={cn(GeistSans.variable)}>
      <body>
        <Providers>
          <div className="min-h-[100dvh]">
            <header className="border-b py-4">
              <nav className="container">
                <Link href="/" className="text-xl font-bold">
                  Bins
                </Link>
              </nav>
            </header>
            <main>{children}</main>
          </div>
          <footer className="border-t py-6">
            <div className="container grid md:grid-cols-2 gap-8">
              <div>
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
                <p>
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
              <div>
                <p className="font-bold text-lg">For Developers</p>
                <ul>
                  <li>
                    <Link href="/docs" rel="noreferrer" className="">
                      About
                    </Link>
                  </li>
                  <li>
                    <Link href="/docs/api" rel="noreferrer" className="">
                      API Reference
                    </Link>
                  </li>
                </ul>
              </div>
            </div>
            <div className="container text-muted-foreground mt-3">
              <CopyRight />
            </div>
          </footer>
        </Providers>
      </body>
    </html>
  );
}
