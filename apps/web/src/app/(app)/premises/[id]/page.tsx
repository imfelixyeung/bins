import { buttonVariants } from "@/components/ui/button";
import { getSummaryAddress } from "@/functions/format-address";
import { searchJobs } from "@/functions/search-jobs";
import AddToRecents from "@/ui/add-to-recents";
import ClientOnly from "@/ui/client-only";
import NearbyMap from "@/ui/nearby-map";
import PremisesJobList from "@/ui/premises-job-list";
import { capitalCase } from "change-case";
import { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import React from "react";
import { z } from "zod";

const premisesIdSchema = z.coerce.number();

export type PageProps = { params: Promise<{ id: string }> };

export const generateMetadata = async (props: PageProps): Promise<Metadata> => {
  const params = await props.params;
  const parsedId = premisesIdSchema.safeParse(params.id);

  if (!parsedId.success) notFound();

  const id = parsedId.data;

  const premises = await searchJobs({ premisesId: id });

  if (!premises) notFound();

  const address = capitalCase(getSummaryAddress(premises));

  return {
    title: `Bin Days for ${address} at ${premises.addressPostcode}`,
    description: `View the scheduled bin collection days for ${address} at ${premises.addressPostcode}`,
    openGraph: {
      title: `Bin Days for ${address} at ${premises.addressPostcode}`,
      description: `View the scheduled bin collection days for ${address} at ${premises.addressPostcode}`,
      images: [
        {
          url: `/api/og/premises/${parsedId}`,
        },
      ],
    },
    twitter: {
      title: `Bin Days for ${address} at ${premises.addressPostcode}`,
      description: `View the scheduled bin collection days for ${address} at ${premises.addressPostcode}`,
      images: [
        {
          url: `/api/og/premises/${parsedId}`,
        },
      ],
    },
  };
};

const Page = async (props: PageProps) => {
  const params = await props.params;

  const { id: _id } = params;

  const parsedId = premisesIdSchema.safeParse(_id);

  if (!parsedId.success) notFound();

  const id = parsedId.data;

  const premises = await searchJobs({ premisesId: id });

  if (!premises) notFound();

  return (
    <div className="container py-16">
      <ClientOnly>
        <AddToRecents premises={premises} />
      </ClientOnly>
      <h1 className="text-3xl font-semibold">Your Bin Day</h1>
      <div className="mt-8">
        <PremisesJobList data={premises} />
      </div>
      <div className="mt-16">
        <Link href="/" className={buttonVariants({ variant: "outline" })}>
          Search for another address
        </Link>
      </div>
      {premises.addressPostcode && (
        <div className="my-16">
          <NearbyMap postcode={premises.addressPostcode} />
        </div>
      )}
    </div>
  );
};

export const revalidate = 3600;

export default Page;
