"use client";

import { buttonVariants, Skeleton, Typography } from "@heroui/react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { usePremisesJobsQuery } from "@/hooks/api";
import AddToRecents from "@/ui/add-to-recents";
import ClientOnly from "@/ui/client-only";
import NearbyMap from "@/ui/nearby-map";
import PremisesJobList from "@/ui/premises-job-list";

const Page = () => {
  const params = useSearchParams();
  const id = params.get("id") ?? "";
  const { isLoading, data: premises } = usePremisesJobsQuery({
    premisesId: Number(id),
  });

  if (isLoading)
    return (
      <div className="container py-16">
        <Typography.Heading>Your Bin Day</Typography.Heading>
        <div className="mt-8">
          <div className="flex justify-between">
            <div className="grow max-w-xs">
              <Typography.Heading level={2} className="mb-3">
                Address
              </Typography.Heading>
              <Skeleton className="mb-6 py-16 w-full" />
            </div>
            <div className="grow max-w-xs">
              <Typography.Heading level={2} className="mb-3">
                At a glance
              </Typography.Heading>
              <Skeleton className="mb-6 py-24.5 w-full" />
            </div>{" "}
          </div>

          <Typography.Heading level={2} className="mb-3">
            By Bin
          </Typography.Heading>
          <Skeleton className="mb-6 py-32" />

          <Typography.Heading level={2} className="mb-3">
            Subscribe
          </Typography.Heading>
          <Skeleton className="mb-6 py-16" />

          <Typography.Heading level={2} className="mb-3">
            Postcode Map
          </Typography.Heading>
          <Skeleton className="mb-6 py-16" />
        </div>
        <div className="mt-16">
          <Link href="/" className={buttonVariants({ variant: "outline" })}>
            Search for another address
          </Link>
        </div>
      </div>
    );

  if (!premises) return null;

  return (
    <div className="container py-16">
      <ClientOnly>
        <AddToRecents premises={premises} />
      </ClientOnly>
      <Typography.Heading>Your Bin Day</Typography.Heading>
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
