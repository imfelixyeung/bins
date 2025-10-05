import React from "react";
import { connection } from "next/server";
import NearbyMap from "@/ui/nearby-map";

const Page = async () => {
  await connection();

  return (
    <div className="container my-16">
      <h1 className="text-3xl font-semibold">Map</h1>

      <NearbyMap />
    </div>
  );
};

export default Page;
