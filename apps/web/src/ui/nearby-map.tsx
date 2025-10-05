import { getNearbyPostcodes } from "@/lib/api/postcodes.io/nearby";
import React from "react";
import NearbyMapClient from "./nearby-map.client";

const NearbyMap = async ({ postcode }: { postcode: string }) => {
  const nearby = await getNearbyPostcodes(postcode).catch(() => null);
  if (!nearby || nearby.length === 0) return null;

  return <NearbyMapClient nearby={nearby} />;
};

export default NearbyMap;
