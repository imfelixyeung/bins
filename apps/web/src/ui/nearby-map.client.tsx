"use client";

import { NearbyPostcode } from "@/lib/api/postcodes.io/nearby";
import React from "react";
import "maplibre-gl/dist/maplibre-gl.css";
import Map, { Marker } from "react-map-gl/maplibre";
import { cn } from "@/lib/utils";

const NearbyMapClient = ({ nearby }: { nearby: NearbyPostcode[] }) => {
  const self = nearby[0]!;
  const { latitude, longitude } = self;

  return (
    <div className="min-h-64 w-full grid rounded-xl overflow-hidden">
      <Map
        initialViewState={{
          longitude,
          latitude,
          zoom: 14,
        }}
        style={{ width: "100%", height: "100%" }}
        mapStyle="https://tiles.openfreemap.org/styles/liberty"
      >
        {nearby.map(({ longitude, latitude, postcode, distance }) => (
          <Marker
            longitude={longitude}
            latitude={latitude}
            anchor="top"
            key={postcode}
          >
            <div
              className={cn(
                "size-3 rounded-full",
                distance === 0 ? "bg-black" : "bg-gray-500"
              )}
            >
              <div
                className={cn(
                  "h-full w-full rounded-full [animation-duration:2.5s]",
                  distance === 0 ? "animate-ping bg-black" : "bg-gray-500"
                )}
              >
                <div className="sr-only">{postcode}</div>
              </div>
            </div>
          </Marker>
        ))}
      </Map>
    </div>
  );
};

export default NearbyMapClient;
