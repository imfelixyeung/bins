"use client";

import { NearbyPostcode } from "@/lib/api/postcodes.io/nearby";
import React, { Fragment, useMemo } from "react";
import "maplibre-gl/dist/maplibre-gl.css";
import Map, { Marker } from "react-map-gl/maplibre";
import { cn } from "@/lib/utils";
import { binStyles, isSupportedBin } from "./bins";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

const NearbyMapClient = ({
  nearby,
}: {
  nearby: (NearbyPostcode & {
    jobs: {
      bin: string;
      date: string;
      postcode: string | null;
    }[];
  })[];
}) => {
  const self = nearby[0]!;
  const { latitude, longitude } = self;

  const uniqueDates = useMemo(() => {
    const dates = new Set<string>();

    nearby.forEach((n) => {
      n.jobs.forEach((job) => {
        dates.add(job.date);
      });
    });

    return Array.from(dates).sort();
  }, [nearby]);

  const [selectedDate, setSelectedDate] = React.useState<string | undefined>(
    uniqueDates[0] || undefined
  );

  return (
    <section className="rounded-xl overflow-hidden">
      <Tabs
        value={selectedDate}
        onValueChange={setSelectedDate}
        className="max-w-full overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      >
        <TabsList>
          {uniqueDates.map((date) => (
            <TabsTrigger value={date} key={date}>
              {new Date(date).toLocaleDateString(undefined, {
                weekday: "short",
                month: "short",
                day: "numeric",
              })}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>
      <div className="min-h-64 w-full grid">
        <Map
          initialViewState={{
            longitude,
            latitude,
            zoom: 14,
          }}
          style={{ width: "100%", height: "100%" }}
          mapStyle="https://tiles.openfreemap.org/styles/liberty"
        >
          {nearby.map(({ longitude, latitude, postcode, distance, jobs }) => {
            const dateJobs = jobs.filter((job) => job.date === selectedDate);
            if (dateJobs.length === 0) return <Fragment key={postcode} />;
            return (
              <Marker
                longitude={longitude}
                latitude={latitude}
                anchor="top"
                key={`${postcode}-${selectedDate}`}
              >
                <div
                  className={cn(
                    "size-3 rounded-full",
                    distance === 0 && "ring-4 ring-black"
                  )}
                >
                  <div className="flex h-full w-full justify-stretch rounded-full overflow-hidden">
                    {dateJobs.map((job) => {
                      const { bin } = job;
                      return (
                        <div
                          key={bin}
                          className={cn(
                            binStyles({
                              bin: isSupportedBin(bin) ? bin : null,
                              styled: false,
                            }),
                            "grow"
                          )}
                        ></div>
                      );
                    })}
                  </div>
                  <div className="sr-only">{postcode}</div>
                </div>
              </Marker>
            );
          })}
        </Map>
      </div>
    </section>
  );
};

export default NearbyMapClient;
