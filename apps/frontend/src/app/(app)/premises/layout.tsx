import { Metadata } from "next";
import { ReactNode } from "react";

const title = "Bin Days";
const description = "View the scheduled bin collection days";
export const metadata: Metadata = {
  title,
  description,
  openGraph: { title, description },
  twitter: { title, description },
};

const Layout = ({ children }: { children: ReactNode }) => {
  return <>{children}</>;
};

export default Layout;
