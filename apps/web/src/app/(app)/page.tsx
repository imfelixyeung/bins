import ClientOnly from "@/ui/client-only";
import PremisesSearchForm from "@/ui/premises-search-form";
import RecentPremises from "@/ui/recent-premises";
import React, { Suspense } from "react";

const Page = () => {
  return (
    <div className="container my-16">
      <section>
        <h1 className="text-3xl font-semibold">Check your bin day</h1>
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
