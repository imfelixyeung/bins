import { parseCsv } from "@/lib/parse-csv";
import { logger } from "@/logger";
import { z } from "zod";
import { createAttributionStatement } from "../attribution-statement";

export const csvUrl =
  "https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv";
export const csvHeaders = [
  "premiseId",
  "addressRoom",
  "addressHouseNumber",
  "addressStreet",
  "addressLocality",
  "addressCity",
  "addressPostCode",
] as const;

const nullableString = z
  .string()
  .nullable()
  .transform((val) => (val === "\x00" ? null : val));

const schema = z.array(
  z.object({
    premiseId: z.string(),
    addressRoom: nullableString,
    addressHouseNumber: nullableString,
    addressStreet: nullableString,
    addressLocality: nullableString,
    addressCity: nullableString,
    addressPostCode: nullableString,
  })
);

export const getHouseholdPremises = async () => {
  const data = await parseCsv({ url: csvUrl, headers: csvHeaders });
  const result = schema.safeParse(data);

  if (!result.success) {
    logger.error(`getWasteCollections failed: ${result.error.message}`);
    return new Error(result.error.message);
  }

  return result.data;
};

export const attributeStatement = createAttributionStatement({
  title: "Household waste collections - Household premises",
  documentUrl: csvUrl,
  yearOfPublication: "2023",
});
