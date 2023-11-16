import { db } from "@/db";
import { premises } from "@/db/schema";
import { eq } from "drizzle-orm";

export const getOne = async (id: string) => {
  const result = await db.query.premises.findFirst({
    where: eq(premises.id, id),
    with: {
      collections: {},
    },
  });

  if (!result) return null;

  return result;
};

export const getManyByPostcode = async (postcode: string) => {
  const result = db.query.premises.findMany({
    where: eq(premises.addressPostCode, postcode),
  });

  return result;
};
