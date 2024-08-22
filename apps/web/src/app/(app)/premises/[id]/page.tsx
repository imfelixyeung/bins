import { buttonVariants } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { getSummaryAddress } from "@/functions/format-address";
import { searchJobs } from "@/functions/search-jobs";
import AddToRecents from "@/ui/add-to-recents";
import ClientOnly from "@/ui/client-only";
import PremisesJobList from "@/ui/premises-job-list";
import { capitalCase } from "change-case";
import { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import React from "react";
import { z } from "zod";

const premisesIdSchema = z.coerce.number();

export type PageProps = { params: { id: string } };

export const generateMetadata = async ({
  params: { id: _id },
}: PageProps): Promise<Metadata> => {
  const id = premisesIdSchema.parse(_id);
  const premises = await searchJobs({ premisesId: id });

  if (!premises) notFound();

  const address = capitalCase(getSummaryAddress(premises));

  return {
    title: `Bin Dates for ${address} at ${premises.addressPostcode}`,
    openGraph: {
      images: [
        {
          url: `/api/og/premises/${id}`,
        },
      ],
    },
  };
};

const Page = async ({ params: { id: _id } }: PageProps) => {
  const parsedId = premisesIdSchema.safeParse(_id);

  if (!parsedId.success) notFound();

  const id = parsedId.data;

  const premises = await searchJobs({ premisesId: id });

  if (!premises) notFound();

  return (
    <div className="container my-16">
      <ClientOnly>
        <AddToRecents premises={premises} />
      </ClientOnly>
      <h1 className="text-3xl font-semibold">Your Bin Day</h1>
      <div className="mt-8">
        <Card className="p-8 max-w-3xl">
          <PremisesJobList data={premises} />
        </Card>
      </div>
      <div className="mt-6">
        <Link href="/" className={buttonVariants({ variant: "outline" })}>
          Search for another address
        </Link>
      </div>
    </div>
  );
};

export default Page;
