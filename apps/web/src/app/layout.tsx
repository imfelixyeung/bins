import { cn } from "@/lib/utils";
import "./globals.css";
import { GeistSans } from "geist/font/sans";

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={cn(GeistSans.variable)}>
      <body>{children}</body>
    </html>
  );
}
