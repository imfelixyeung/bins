import ClientOnly from "@/ui/client-only";
import PremisesSearchForm from "@/ui/premises-search-form";
import RecentPremises from "@/ui/recent-premises";
import React, { Suspense } from "react";

const Page = () => {
  return (
    <div className="container py-24">
      <section className="py-16 flex flex-col items-center justify-center text-center gap-3">
        <h1 className="text-3xl md:text-5xl font-semibold text-balance">
          Check Your Bin Days
        </h1>
        <p className="text-muted-foreground text-xl">
          Check when your bins are scheduled to be emptied.
        </p>
      </section>
      <section className="py-16">
        <h2 className="text-2xl font-semibold">Search for your address</h2>
        <div className="mt-8">
          <div className="max-w-3xl">
            <Suspense>
              <PremisesSearchForm />
            </Suspense>
          </div>
        </div>
      </section>

      <ClientOnly>
        <RecentPremises />
      </ClientOnly>
    </div>
  );
};

export default Page;
