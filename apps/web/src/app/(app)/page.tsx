import PremisesSearchForm from "@/ui/premises-search-form";
import React, { Suspense } from "react";

const Page = () => {
  return (
    <div className="container py-24">
      <section className="py-16 flex flex-col items-center justify-center gap-3">
        <h1 className="text-3xl md:text-5xl font-semibold text-balance text-center">
          Check Your Bin Days
        </h1>
        <p className="text-muted-foreground text-xl text-center text-balance">
          Check when your bins are scheduled to be emptied.
        </p>
        <Suspense>
          <div className="w-full flex justify-center mt-16">
            <PremisesSearchForm />
          </div>
        </Suspense>
      </section>
    </div>
  );
};

export default Page;
