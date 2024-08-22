"use client";

import { useSavedPremises } from "@/hooks/use-saved-premises";
import Link from "next/link";
import React from "react";
import Address from "./address";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

const RecentPremises = () => {
  const { premises, removeAll } = useSavedPremises();

  if (premises.length === 0) return null;

  return (
    <>
      <section className="mt-16">
        <h2 className="text-2xl font-semibold">Recent Searches</h2>
        <div className="mt-3">
          <div className="flex gap-3">
            {premises.map((premises) => {
              return (
                <div key={premises.id}>
                  <Card>
                    <Link href={`/premises/${premises.id}`}>
                      <CardContent className="pt-6">
                        <Address data={premises} />
                      </CardContent>
                    </Link>
                  </Card>
                </div>
              );
            })}
          </div>
          <div className="mt-3">
            <Button variant="outline" onClick={removeAll} type="button">
              Clear Recent Searches
            </Button>
          </div>
        </div>
      </section>
    </>
  );
};

export default RecentPremises;
