"use client";

import { buttonVariants } from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import AddToRecents from "@/ui/add-to-recents";
import ClientOnly from "@/ui/client-only";
import NearbyMap from "@/ui/nearby-map";
import PremisesJobList from "@/ui/premises-job-list";
import { searchJobs } from "@/v2api/client";

const Page = () => {
  const params = useSearchParams();
  const id = params.get("id") ?? "";
  const jobsQuery = useQuery({
    queryKey: ["searchJobs", id],
    queryFn: () => searchJobs({ premisesId: Number(id) }),
  });

  if (!jobsQuery.data) return null;

  const premises = jobsQuery.data;

  return (
    <div className="container py-16">
      <ClientOnly>
        <AddToRecents premises={premises} />
      </ClientOnly>
      <h1 className="text-3xl font-semibold">Your Bin Day</h1>
      <div className="mt-8">
        <PremisesJobList data={premises} />
      </div>
      {premises.addressPostcode && (
        <NearbyMap postcode={premises.addressPostcode} />
      )}
      <div className="mt-16">
        <Link href="/" className={buttonVariants({ variant: "outline" })}>
          Search for another address
        </Link>
      </div>
    </div>
  );
};

export default Page;
