import { parseCsv } from "@/lib/parse-csv";
import { z } from "zod";
import { createAttributionStatement } from "../attribution-statement";
import { logger } from "@/logger";

export const csvUrl =
  "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv";
export const csvHeaders = ["premiseId", "binColour", "date"] as const;

const schema = z.array(
  z.object({
    premiseId: z.string(),
    binColour: z.string(),
    date: z.string(),
  })
);

export const getWasteCollections = async () => {
  const data = await parseCsv({ url: csvUrl, headers: csvHeaders });
  const result = schema.safeParse(data);

  if (!result.success) {
    logger.error(`getWasteCollections failed: ${result.error.message}`);
    return new Error(result.error.message);
  }

  return result.data;
};

export const attributeStatement = createAttributionStatement({
  title: "Household waste collections - Waste collections",
  documentUrl: csvUrl,
  yearOfPublication: "2023",
});
