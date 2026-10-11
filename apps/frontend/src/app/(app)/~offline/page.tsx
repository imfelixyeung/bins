import { buttonVariants, Typography } from "@heroui/react";
import type { Metadata } from "next";
import Link from "next/link";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Offline",
};

export default function Page() {
  return (
    <div className="container py-24 flex justify-center items-center flex-col">
      <Typography.Heading>Bins Offline</Typography.Heading>
      <Typography.Heading level={2} className="text-xl mt-3">
        Please check your internet connection
      </Typography.Heading>

      <Link href="/" className={cn(buttonVariants(), "mt-6")}>
        Return home
      </Link>
    </div>
  );
}
