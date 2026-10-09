"use client";

import { useSavedPremises } from "@/hooks/use-saved-premises";
import Link from "next/link";
import React from "react";
import { Button, Surface } from "@heroui/react";
import { MapPinHouseIcon, XIcon } from "lucide-react";
import { getTwoLineFullAddress } from "@/functions/format-address";

const RecentPremises = () => {
  const { premises, remove, removeAll } = useSavedPremises();

  if (premises.length === 0) return null;

  return (
    <>
      <section className="mt-16">
        <h2 className="text-lg font-medium text-center">Recents</h2>
        <div className="mt-3">
          <div className="flex gap-3 flex-col">
            {premises.map((premises) => {
              const [line1, line2] =
                getTwoLineFullAddress(premises).split("\n");
              return (
                <div key={premises.id}>
                  <Surface
                    className="flex gap-3 items-center px-3 py-3 relative hover:shadow-md transition-shadow group border rounded-3xl"
                    variant="secondary"
                  >
                    <div className="size-10 rounded-full flex items-center justify-center border shrink-0 bg-linear-to-b from-foreground/0 to to-foreground/10">
                      <MapPinHouseIcon size={20} />
                    </div>
                    <Link
                      href={`/premises?id=${premises.id}`}
                      className="after:absolute after:inset-0 grow"
                      prefetch
                    >
                      <h3 className="font-medium line-clamp-2">{line1}</h3>
                      <p className="text-sm line-clamp-2 text-muted">{line2}</p>
                    </Link>
                    <button
                      className="p-2 opacity-50 md:opacity-30 group-hover:opacity-100 transition-opacity z-10 hover:scale-110"
                      title="remove from recents"
                      onClick={() => remove(premises.id)}
                    >
                      <div className="sr-only">Remove</div>
                      <XIcon size={16} />
                    </button>
                  </Surface>
                </div>
              );
            })}
          </div>
          <div className="mt-3">
            <Button variant="secondary" onClick={removeAll} type="button">
              Clear Recent Searches
            </Button>
          </div>
        </div>
      </section>
    </>
  );
};

export default RecentPremises;
