import { parseCsv } from "@/lib/parse-csv";
import { z } from "zod";
import { createAttributionStatement } from "../attribution-statement";
import { logger } from "@/logger";

export const csvUrl =
  "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv";
export const csvHeaders = ["premiseId", "binColour", "date"] as const;

export const getWasteCollections = async () => {
  const data = await parseCsv({ url: csvUrl, headers: csvHeaders });
  return data;
};

export const attributeStatement = createAttributionStatement({
  title: "Household waste collections - Waste collections",
  documentUrl: csvUrl,
  yearOfPublication: "2023",
});
