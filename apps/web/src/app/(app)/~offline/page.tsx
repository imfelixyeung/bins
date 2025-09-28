import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Offline",
};

export default function Page() {
  return (
    <div className="container py-24 flex justify-center items-center flex-col">
      <h1 className="text-3xl font-semibold">Bins Offline</h1>
      <h2 className="text-xl mt-3">Please check your internet connection</h2>

      <Link href="/" className={cn(buttonVariants(), "mt-6")}>
        Return home
      </Link>
    </div>
  );
}
