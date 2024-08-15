import { buttonVariants } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { searchJobs } from "@/functions/search-jobs";
import PremisesJobList from "@/ui/premises-job-list";
import Link from "next/link";
import { notFound } from "next/navigation";
import React from "react";
import { z } from "zod";

const premisesIdSchema = z.coerce.number();

const Hello = async ({ params: { id: _id } }: { params: { id: string } }) => {
  const parsedId = premisesIdSchema.safeParse(_id);

  if (!parsedId.success) notFound();

  const id = parsedId.data;

  const premises = await searchJobs({ premisesId: id });

  if (!premises) notFound();

  return (
    <div className="container my-16">
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

export default Hello;
