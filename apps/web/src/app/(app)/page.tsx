import { Card } from "@/components/ui/card";
import PremisesSearchForm from "@/ui/premises-search-form";
import React, { Suspense } from "react";

const Page = () => {
  return (
    <div className="container my-16">
      <h1 className="text-3xl font-semibold">Check your bin day</h1>
      <div className="mt-8">
        <Card className="p-8 max-w-3xl">
          <Suspense>
            <PremisesSearchForm />
          </Suspense>
        </Card>
      </div>
    </div>
  );
};

export default Page;
